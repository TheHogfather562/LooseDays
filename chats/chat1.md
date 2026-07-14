# Design brief for app screens

_Started 2026-07-14 19:58 UTC_

---

## User

<system-info comment="Only acknowledge these if relevant">
Project title is now "Build the frontend for a web app called **Loose Days**. Mobile-responsive web ap"
The project currently has no files except possibly dotfiles — a `list_files` on the project root would return nothing else.
Current date is now July 14, 2026
</system-info>

<default aesthetic_system_instructions>
The user has not attached a design system. If they have ALSO not attached references or art direction, and the project is empty, you must ASK the user what visual aesthetic they want. Use the questions_v2 tool to ask about preferred vibe, audience, colors, type, mood, etc. Do NOT just pick your own visual aesthetic without getting the user's aesthetic input -- this is how you get slop!

Once answered, use this guidance when creating designs:
- Choose a type pairing from web-safe set or Google Fonts. Helvetica is a good choice. Avoid hard-to-read or overly stylized fonts. Use 1-3 fonts only.
- Foreground and background: choose a color tone (warm, cool, neutral, something in-between). Use subtly-toned whites and blacks; avoid saturations above 0.02 for whites.
- Accents: choose 0-2 additional accent colors using oklch. All accents should share same chroma and lightness; vary hue.
- NEVER write out an SVG yourself that's more complicated than a square, circle, diamond, etc.
- For imagery, never hand-draw SVGs; use subtly-striped SVG placeholders instead with monospace explainers for what should be dropped there (e.g. “product shot”)

CRITICAL: ignore default aesthetic entirely if given other aesthetic instructions like reference images, design systems or guidance, or if there are files in the project already.
</default aesthetic_system_instructions>

<pasted_text name="Pasted text (45 lines)">
Build the frontend for a web app called **Loose Days**. Mobile-responsive web app (not native), SvelteKit, Tailwind. Full spec context below — read it all before starting, it defines the data model and flows precisely.

## What the app is
A social calendar for friends. No in-app messaging — every invite/nudge hands off to the user's own SMS or WhatsApp via a deep link (`sms:`/`wa.me`), then brings them back into the app. Two coexisting mechanisms:
1. **Personal calendars + access requests** — each user maintains a standing free/busy/maybe calendar. Friends request access to view it (one-off date range, or ongoing), the owner approves/denies and picks how much detail to reveal (full status, or just mutual-overlap days).
2. **Group polls** — create a date range, invite people (users or not-yet-users), everyone marks free/busy/maybe for that range, results shown as a days × people grid with the best-overlap range highlighted.

Auth is email magic link (no password, no SMS OTP). Sign-up is invite-only in v1 — phone number is entered as an identifier (used for friend-matching, not verified), but there's no open public sign-up flow to design for.

## Screens to build
1. **Sign in** — email input → "check your email" state → (magic link lands them authenticated, no code-entry UI needed).
2. **Onboarding / add friends** — enter/import contacts, see which numbers match existing users, send "join Loose Days" invite via `sms:`/`wa.me` deep link for non-matches.
3. **My calendar** — the standing personal calendar. Tap a day to cycle/set free / busy / maybe, optional short note. Month view primarily, this is the home screen.
4. **Friends list** — friends, with entry points to (a) request access to their calendar, (b) see calendars they've already granted you access to.
5. **Request calendar access** — pick a friend, pick scope (specific range vs. ongoing), send request (triggers `sms:`/`wa.me` deep link to notify them).
6. **Incoming access requests** — approve/deny queue. On approve, choose detail level (full vs. overlap-only) before confirming.
7. **"Who can see my calendar"** — management screen listing everyone with standing access, with a one-tap revoke.
8. **View a friend's calendar** — read-only view at whatever detail level was granted (full status+notes, or just highlighted mutual-free days).
9. **Create a poll** — date range picker, invitee picker (friends + raw phone numbers), optional title/note, then send invites via deep link per invitee (or one bulk share sheet).
10. **Respond to a poll** — day-by-day free/busy/maybe entry for the poll's range (works for non-users via a token link too — no forced sign-up).
11. **Poll results** — grid of days × people, colour-coded, best-overlap range highlighted.

## Design direction
- Credibility over warmth — clean, confident, a bit understated. Not cutesy, not corporate-generic Tailwind-default either. Think: something that looks like a tool a competent engineer built for their friends, not a consumer social app trying to look fun.
- Mobile-first — most usage will be on a phone screen. Calendar/grid views need to work at narrow widths without cramming.
- The free/busy/maybe distinction should be visually unambiguous at a glance (colour + not relying on colour alone — some people are colourblind).
- Read `/mnt/skills/public/frontend-design/SKILL.md` before starting for house style/typography/token guidance rather than defaulting to generic component patterns.

## Data model (Postgres — build against this shape; use mock/local data for now, no live backend yet)
```
users: id, phone_hash, email, display_name, created_at
friend_edges: user_id, friend_id, created_at
calendar_days: user_id, date, availability(free/busy/maybe), note, updated_at
calendar_access_requests: id, requester_id, owner_id, scope(range/standing), range_start, range_end, detail_level(full/overlap_only), status(pending/approved/denied/revoked), created_at, responded_at
polls: id, creator_id, title, range_start, range_end, created_at
poll_invitees: id, poll_id, user_id(nullable), phone_hash(nullable), access_token, status(invited/responded)
poll_responses: poll_id, invitee_id, date, availability, note
```

## Explicit constraints
- No in-app chat/messaging UI of any kind — every "contact this person" action is a deep link out to their own SMS/WhatsApp app, then the user returns to Loose Days.
- No password field anywhere — auth is magic link only.
- Build against mock data / local state for now — there's no live API yet, just get the UI and flows right. Structure data-fetching so swapping in real API calls later is a small change, not a rewrite.

Start with the personal calendar screen and the poll creation/response/results flow — those are the two screens most worth getting right first. Ask me before assuming details not covered above.
</pasted_text>

<!-- The user explicitly selected the following skills for this project, as attachments to their message. These are not optional context — they define how you work. Use them. -->
<attached-skill name="Design Components">
This project uses Design Components: every design is a single streaming `Name.dc.html` file. The full authoring spec is in your system prompt under "Writing code — Design Components" — follow it. Author and edit `.dc.html` content with the `dc_write`, `dc_html_str_replace`, `dc_js_str_replace`, and `dc_set_props` tools (not `write_file`; `str_replace_edit` works but won't stream); template edits stream into the live preview as you type.
</attached-skill>

<attached-skill name="Hi-fi design">
Create a high-fidelity, polished design.

Follow this general design process (use the todo list to remember):
(1) ask questions, (2) find existing UI kits and collect design context — copy ALL relevant components and read ALL relevant examples; ask the user if you can't find them, (3) start your file with assumptions + context + design reasoning (as if you are a junior designer and the user is your manager), with placeholders for the designs, and show it to the user early, (4) build out the designs and show the user again ASAP; append some next steps, (5) use your tools to check, verify and iterate on the design.

Good hi-fi designs do not start from scratch — they are rooted in existing design context. Ask the user to Import their codebase, or find a suitable UI kit / design resources, or ask for screenshots of existing UI. You MUST spend time trying to acquire design context, including components. If you cannot find them, ask the user for them. In the Import menu, they can link a local codebase, provide screenshots or Figma links; they can also link another project. Mocking a full product from scratch is a LAST RESORT and will lead to poor design. If stuck, try listing design assets and ls'ing design system files — be proactive! Some designs may need multiple design systems — get them all. Use the starter components (device frames and the like) to get high-quality scaffolding for free.

When showing multiple design options on one page, decide between (a) a single full-size responsive prototype with a tweaks panel, or (b) a vertical stack of anchored option cards. Choose based on how design-y vs prototype-y the ask is, how many options there are, and how big each is. For (b):

Present multiple design options as a vertical stack of turns — each turn of options is its own `<section>`, newest turn at the **top**, and every option gets a stable `{turn}{letter}` id (`1a`, `1b`, `2a`…) that the user references back in chat and you cross-link between turns. Always include `<meta name="design_doc_mode" content="canvas">` in `<helmet>` — the host provides pan/zoom, so the user can freely zoom out on designs wider than the viewport.

**How to write it** — put one `<style>` block in `<helmet>`, then one `<section class="dv-turn">` per turn as a **direct child of the root** (right after `</helmet>`, no wrapper). When the user asks for another round, **insert the new section ABOVE the existing ones** so the latest work sits at the top; never reorder, renumber, or delete earlier turns.

```html
<helmet data-dc-atomics><meta name="design_doc_mode" content="canvas"><style>body{margin:0;background:#f0eee9;font-family:system-ui,sans-serif}.dv-turn{padding:40px 44px 32px;border-bottom:1px solid rgba(0,0,0,.08);scroll-margin-top:16px}.dv-thd{display:flex;align-items:baseline;gap:10px;margin:0 0 20px}.dv-tid{font:600 10px ui-monospace,Menlo,monospace;padding:3px 7px;background:#1a1a1a;color:#fff;border-radius:4px;text-decoration:none}.dv-tname{font:600 13px/1.2 system-ui,sans-serif;color:#1a1a1a}.dv-opts{display:flex;flex-wrap:wrap;gap:28px;align-items:flex-start}.dv-opt{flex:none;display:flex;flex-direction:column;gap:9px;scroll-margin-top:16px}.dv-oid{font:600 10.5px ui-monospace,Menlo,monospace;padding:3px 7px;background:rgba(0,0,0,.08);color:#1a1a1a;border-radius:5px;text-decoration:none}.dv-olabel{display:flex;align-items:baseline;gap:8px;font:400 11px/1.3 system-ui,sans-serif;color:rgba(0,0,0,.55)}.dv-card{max-width:100%;background:#fff;border:1px solid rgba(0,0,0,.08);border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,.06);overflow:hidden}.dv-opt:target .dv-oid{background:#2a78d6;color:#fff}.dv-next{margin:22px 0 0;font:12px/1.5 system-ui,sans-serif;color:rgba(0,0,0,.5)}</style></helmet>
<section class="dv-turn" id="t2">
<div class="dv-thd"><a class="dv-tid" href="#t2">2</a><span class="dv-tname">Riffs on <a class="dv-oid" href="#1b">1b</a></span></div>
<div class="dv-opts">
<div class="dv-opt" id="2a"><div class="dv-olabel"><a class="dv-oid" href="#2a">2a</a>Tighter spacing</div><div class="dv-card" style="width:360px">…design…</div></div>
<div class="dv-opt" id="2b">…</div>
</div>
<p class="dv-next">Try next: "more like <a class="dv-oid" href="#2a">2a</a> but with the serif from <a class="dv-oid" href="#1c">1c</a>" · "make <a class="dv-oid" href="#2b">2b</a> full-bleed" · "new directions"</p>
</section>
<section class="dv-turn" id="t1">…turn 1, unchanged…</section>
```

**Rules:** turn section ids are `t1`, `t2`, `t3`…; option ids are `1a`, `1b`, `2a`… and go on the option's **outermost** element (`.dv-opt`), never on the badge — so `#1b` scrolls the whole option into view. Ids are stable forever, never reused or renumbered. Options within a turn sit side-by-side in a wrapping row; don't hand-roll your own pan/zoom — the host canvas provides it. **Every** option-id reference in the file — turn heading, option label, `.dv-next` line, any prose — is an `<a class="dv-oid" href="#1b">1b</a>` link, never a bare `1b`; in your chat replies, just write `1b`. End each turn with a one-line `.dv-next` of 2–3 plain-English follow-ups the user could paste into chat. Size each `.dv-card` to its content (explicit width is fine); don't use `height:100%`.

When designing, asking many good questions is ESSENTIAL.

Give options: try to give 3+ variations across several dimensions. Mix by-the-book designs that match existing patterns with new and novel interactions, including interesting layouts, metaphors, and visual styles. Have some options that use color or advanced CSS; some with iconography and some without. Start your variations basic and get more advanced and creative as you go! Try remixing the brand assets and visual DNA in interesting ways — play with scale, fills, texture, visual rhythm, layering, novel layouts, type treatments. The goal is not the perfect option; it's exploring atomic variations the user can mix and match.

CSS, HTML, JS and SVG are amazing. Users often don't know what they can do. Surprise the user.

If you do not have an icon, asset or component, draw a placeholder: in hi-fi design, a placeholder is better than a bad attempt at the real thing.
</attached-skill>

<attached-skill name="Interactive prototype">
Create a fully interactive prototype with realistic state management and transitions. Use React useState/useEffect for dynamic behavior. Include hover states, click interactions, form validation, animated transitions, and multi-step navigation flows. It should feel like a real working app, not a static mockup.
</attached-skill>


<system-reminder>Auto-injected reminder (ignore if not relevant): do not recreate copyrighted or branded UI unless the user's email domain matches that company. Create original designs instead.</system-reminder>

## Assistant

_[tool: read_skill_prompt]_

_[tool: questions_v2]_

## User

Questions answered:
- vibe: Explore a few options
- theme: Let me pick per-screen
- color_temp: Warm neutrals (stone/sand)
- accent: 0
- fbm_colors: Decide for me
- typeface: Decide for me
- density: Airy — generous spacing, fewer things per screen
- scope: Just the 2 priority screens (My Calendar + full poll flow: create/respond/results)
- nav: Bottom tab bar (Calendar / Friends / Polls)
- variations: Show me 2-3 directions for the calendar screen first

## User

Found issues — fixing…

## User

Found issues — fixing…

## Assistant

_[tool: dc_html_str_replace]_

_[tool: dc_html_str_replace]_

_[tool: dc_html_str_replace]_

_[tool: dc_html_str_replace]_

_[tool: ready_for_verification]_

Bumped card height to 760px and made the card a flex column so the tab bar sits fixed at bottom — checking with the verifier now.

## User

Lets move forward with 1b

## Assistant

_[tool: write_file]_

_[tool: dc_write]_

_[tool: ready_for_verification]_

Built My Calendar + full poll flow (list → create → send invites → respond → results) in 1b's style, with bottom tab bar (Friends stubbed for next pass). Waiting on verifier.

## User

Lets add a default day status which is unassigned so making something as free is a active and positive move

## Assistant

_[tool: dc_js_str_replace]_

_[tool: dc_js_str_replace]_

_[tool: dc_js_str_replace]_

Now add a "Clear" action in the day panel and update the legend to include an "unassigned" swatch.

_[tool: dc_html_str_replace]_

_[tool: dc_html_str_replace]_

_[tool: grep]_

My earlier replace already fixed line 61 (calendar legend), good — that's the one I meant to update, already done. The results-screen legend (line 258-259) already has "pending" so no change needed there.

_[tool: dc_js_str_replace]_

_[tool: ready_for_verification]_

Days now default to unassigned (small gray dot, no status) instead of implicit "free" — marking Free is now an explicit, active choice with its own green ring. Added a Clear action and "not set" to the legend.

## User

Looks great how does it work on a desktop view as well?

## Assistant

_[tool: show_html]_

On desktop it renders as a centered mobile-width column (max 480px) on the stone background, like a phone card sitting in the browser — the app itself stays a single mobile-sized layout rather than reflowing into a desktop grid, since this is built mobile-first per the spec. Everything (calendar, poll flow) still works via mouse clicks. If you'd rather have a wider desktop layout (e.g. calendar + poll list side by side above some breakpoint), let me know and I can add that.

## User

Please create all the screens in the initial prompt in this style

## Assistant

_[tool: read_file]_

_[tool: str_replace_edit]_

_[tool: dc_html_str_replace]_

_[tool: dc_html_str_replace]_

Now replace the Friends tab stub with the real friends/access screens.

_[tool: dc_html_str_replace]_

_[tool: dc_js_str_replace]_

Now add the new methods before `renderVals()`.

_[tool: dc_js_str_replace]_

_[tool: dc_js_str_replace]_

_[tool: dc_js_str_replace]_

Now verify it renders cleanly across the new flow.

_[tool: ready_for_verification]_

Added all remaining screens in the same style: Sign in (email → check-email state), Onboarding (contact matching + SMS/WhatsApp invites), Friends list with Request access / Incoming requests (approve w/ detail-level choice) / Who-can-see-me (revoke) / View a friend's calendar (full vs overlap-only rendering). Flow: sign in → onboarding → main app (tab bar). Checking with the verifier now.

## User

how to I log into the example?

## Assistant

There's no real backend — it's a mock-data prototype. On the Sign in screen, type any text with an "@" in the email field and tap "Send magic link"; on the "Check your email" screen that follows, tap the small "Continue (demo) →" link at the bottom to skip the fake email step and proceed into onboarding → the app.

## User

How does someone request another week if they already have access to one week?

## Assistant

_[tool: str_replace_edit]_

_[tool: dc_html_str_replace]_

_[tool: dc_js_str_replace]_

That was a real gap: previously any granted access permanently hid "Request access." Now a friend who only gave you a one-off range (like Marcus's overlap access through Jul 31) shows both "View calendar" and a "Request another range" link, so you can ask for a different week without waiting for the old grant to lapse. Standing (ongoing) grants like Priya's don't show that link since you already have continuous access.

_[tool: ready_for_verification]_

Fixed — Friends list now supports requesting an additional range even after having range-limited access.

