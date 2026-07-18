use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

macro_rules! text_enum {
	($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
		#[serde(rename_all = "snake_case")]
		pub enum $name {
			$($variant),+
		}

		impl $name {
			pub fn as_str(&self) -> &'static str {
				match self {
					$(Self::$variant => $text),+
				}
			}
		}

		impl TryFrom<&str> for $name {
			type Error = AppError;
			fn try_from(s: &str) -> AppResult<Self> {
				match s {
					$($text => Ok(Self::$variant),)+
					other => Err(AppError::Internal(anyhow::anyhow!("unknown {} value: {other}", stringify!($name)))),
				}
			}
		}

		impl TryFrom<String> for $name {
			type Error = AppError;
			fn try_from(s: String) -> AppResult<Self> {
				Self::try_from(s.as_str())
			}
		}
	};
}

text_enum!(Availability { Free => "free", Busy => "busy", Maybe => "maybe" });
text_enum!(AccessScope { Range => "range", Standing => "standing" });
text_enum!(DetailLevel { Full => "full", OverlapOnly => "overlap_only" });
text_enum!(InviteeStatus { Invited => "invited", Responded => "responded" });

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUserDto {
	pub id: Uuid,
	pub display_name: String,
	pub email: String,
	/// Only the hash is ever stored server-side, so this is a presence
	/// signal for the UI ("have I set a phone yet?"), never the number.
	pub phone_set: bool,
	#[serde(skip)]
	pub onboarded: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendDto {
	pub id: Uuid,
	pub display_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactDto {
	pub id: String,
	pub name: String,
	pub phone: String,
	pub matched: bool,
	pub user_id: Option<Uuid>,
	pub already_friend: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FriendAccessGrantDto {
	pub level: String, // "full" | "overlap"
	pub scope: AccessScope,
	pub range_start: Option<NaiveDate>,
	pub range_end: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDayEntryDto {
	pub status: Option<Availability>,
	pub note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingRequestDto {
	pub id: Uuid,
	pub requester_id: Uuid,
	pub requester_name: String,
	pub scope: AccessScope,
	pub range_start: Option<NaiveDate>,
	pub range_end: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandingAccessGrantDto {
	pub id: Uuid,
	pub friend_id: Uuid,
	pub friend_name: String,
	pub detail_level: DetailLevel,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollInviteeDto {
	pub id: Uuid,
	pub user_id: Option<Uuid>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub phone: Option<String>,
	pub name: String,
	pub status: InviteeStatus,
	pub access_token: Option<String>,
	pub is_me: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PollDto {
	pub id: Uuid,
	pub title: String,
	pub note: String,
	pub creator_id: Uuid,
	pub range_start: NaiveDate,
	pub range_end: NaiveDate,
	pub invitees: Vec<PollInviteeDto>,
	pub responses: HashMap<Uuid, HashMap<NaiveDate, Availability>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePollInput {
	pub title: String,
	pub note: String,
	pub start: NaiveDate,
	pub end: NaiveDate,
	pub friend_ids: Vec<Uuid>,
	/// Replaces the old `phoneChips: string[]` — a name is required per
	/// invitee now that the server never stores the raw phone number, so it
	/// can't fall back to using the phone digits as the display name.
	pub phone_invitees: Vec<PhoneInviteeInput>,
}

#[derive(Debug, Deserialize)]
pub struct PhoneInviteeInput {
	pub phone: String,
	pub name: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PasskeyDto {
	pub id: Uuid,
	pub label: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
	pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}
