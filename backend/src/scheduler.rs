use std::time::Duration as StdDuration;

use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeZone, Utc, Weekday};
use chrono_tz::Tz;

use crate::email::send_weekly_nudge_email;
use crate::repo::nudges;
use crate::state::AppState;

/// Spawns the background task that emails users a Sunday reminder to fill in
/// their availability for the coming week. A no-op when disabled via config.
pub fn spawn(state: AppState) {
	if !state.config.weekly_nudge_enabled {
		tracing::info!("[nudge] weekly nudge disabled");
		return;
	}
	tokio::spawn(run(state));
}

async fn run(state: AppState) {
	let tz = state.config.weekly_nudge_tz;
	let hour = state.config.weekly_nudge_hour;
	let minute = state.config.weekly_nudge_minute;

	// Startup catch-up: if the process was down over this Sunday's send time,
	// fire once now. The weekly_nudges table makes this idempotent, so a
	// restart shortly after a successful send never re-mails anyone.
	let now = Utc::now().with_timezone(&tz);
	if now.weekday() == Weekday::Sun {
		if let Some(send_at) = at_time(now.date_naive(), hour, minute, tz) {
			if now >= send_at {
				send_batch(&state).await;
			}
		}
	}

	loop {
		let now = Utc::now().with_timezone(&tz);
		let Some(next) = next_send_time(now, hour, minute, tz) else {
			tracing::error!("[nudge] could not compute next send time; retrying in 1h");
			tokio::time::sleep(StdDuration::from_secs(3600)).await;
			continue;
		};
		let wait = (next - now).to_std().unwrap_or(StdDuration::from_secs(60));
		tracing::info!("[nudge] next weekly nudge at {next}");
		tokio::time::sleep(wait).await;

		send_batch(&state).await;

		// Step past the send minute so the next `next_send_time` doesn't resolve
		// back to the same instant if the sleep returned a touch early.
		tokio::time::sleep(StdDuration::from_secs(90)).await;
	}
}

/// Loads the users due a nudge for the coming week and emails each one,
/// recording the send so it isn't repeated.
async fn send_batch(state: &AppState) {
	let today = Utc::now().with_timezone(&state.config.weekly_nudge_tz).date_naive();
	// "The following week" relative to a Sunday send is the Mon–Sun that starts
	// the very next day.
	let week_start = next_monday(today);
	let week_end = week_start + Duration::days(6);

	let users = match nudges::users_to_nudge(&state.pool, week_start, week_end).await {
		Ok(u) => u,
		Err(e) => {
			tracing::error!("[nudge] failed to load users to nudge: {e:?}");
			return;
		}
	};

	tracing::info!(
		"[nudge] sending weekly nudge to {} user(s) for week of {week_start}",
		users.len()
	);

	let calendar_url = format!("{}/calendar", state.config.public_origin);
	for u in users {
		send_weekly_nudge_email(&state.config, &u.email, &u.display_name, &calendar_url, week_start).await;
		if let Err(e) = nudges::record_nudge(&state.pool, u.id, week_start).await {
			tracing::error!("[nudge] failed to record nudge for {}: {e:?}", u.id);
		}
	}
}

/// Resolves `date` at `hour:minute` in `tz` to a concrete instant, stepping
/// past a DST spring-forward gap if that wall-clock time doesn't exist.
fn at_time(date: NaiveDate, hour: u32, minute: u32, tz: Tz) -> Option<DateTime<Tz>> {
	let naive = date.and_hms_opt(hour, minute, 0)?;
	match tz.from_local_datetime(&naive) {
		LocalResult::Single(dt) => Some(dt),
		LocalResult::Ambiguous(dt, _) => Some(dt),
		LocalResult::None => tz.from_local_datetime(&(naive + Duration::hours(1))).single(),
	}
}

/// The next Sunday-at-`hour:minute` strictly after `now` (today included when
/// its send time is still ahead).
fn next_send_time(now: DateTime<Tz>, hour: u32, minute: u32, tz: Tz) -> Option<DateTime<Tz>> {
	let today = now.date_naive();
	for add in 0..=7 {
		let d = today + Duration::days(add);
		if d.weekday() == Weekday::Sun {
			if let Some(send_at) = at_time(d, hour, minute, tz) {
				if send_at > now {
					return Some(send_at);
				}
			}
		}
	}
	None
}

/// The first Monday strictly after `date` (i.e. the start of the next week).
fn next_monday(date: NaiveDate) -> NaiveDate {
	let ahead = 7 - date.weekday().num_days_from_monday() as i64;
	let ahead = if ahead == 0 { 7 } else { ahead };
	date + Duration::days(ahead)
}

#[cfg(test)]
mod tests {
	use super::*;
	use chrono::Timelike;

	#[test]
	fn next_monday_from_sunday_is_tomorrow() {
		// 2026-07-26 is a Sunday.
		let sun = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
		assert_eq!(sun.weekday(), Weekday::Sun);
		assert_eq!(next_monday(sun), NaiveDate::from_ymd_opt(2026, 7, 27).unwrap());
	}

	#[test]
	fn next_monday_from_monday_is_next_week() {
		let mon = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
		assert_eq!(mon.weekday(), Weekday::Mon);
		assert_eq!(next_monday(mon), NaiveDate::from_ymd_opt(2026, 8, 3).unwrap());
	}

	#[test]
	fn next_send_time_lands_on_a_sunday() {
		let tz = chrono_tz::UTC;
		// A Wednesday.
		let now = tz.with_ymd_and_hms(2026, 7, 22, 12, 0, 0).unwrap();
		let next = next_send_time(now, 18, 0, tz).unwrap();
		assert_eq!(next.weekday(), Weekday::Sun);
		assert_eq!(next.date_naive(), NaiveDate::from_ymd_opt(2026, 7, 26).unwrap());
		assert_eq!(next.hour(), 18);
	}

	#[test]
	fn next_send_time_skips_today_once_time_has_passed() {
		let tz = chrono_tz::UTC;
		// Sunday, after the 18:00 send time.
		let now = tz.with_ymd_and_hms(2026, 7, 26, 19, 0, 0).unwrap();
		let next = next_send_time(now, 18, 0, tz).unwrap();
		assert_eq!(next.date_naive(), NaiveDate::from_ymd_opt(2026, 8, 2).unwrap());
	}
}
