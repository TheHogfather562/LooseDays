# Loose Days

A social calendar for friends. No in-app messaging — every invite or nudge
hands off to your own SMS/WhatsApp app, then brings you back into Loose Days.

Two ways to coordinate:

- **Personal calendars + access requests** — everyone keeps a standing
  free/busy/maybe calendar. Friends request access (a one-off date range, or
  ongoing), and the owner approves/denies and picks how much detail to reveal
  (full status, or just mutual-overlap days).
- **Group polls** — pick a date range, invite people (friends or raw phone
  numbers — no account required to respond), everyone marks their
  availability, and results show as a days × people grid with the best
  overlap highlighted.

Auth is email magic link (no passwords). Sign-up is invite-only: the first
account is bootstrapped from an env var, and every other person has to be
invited from inside the app. Phone numbers are only ever stored as a salted
hash (used to match friends against each other), never in the clear.

## Architecture

Two separate pieces, deployed independently:

- **Frontend** (`src/`) — SvelteKit + Tailwind, renders entirely client-side
  (`ssr = false`) and talks to the backend purely over `fetch`. Builds to a
  static SPA bundle (`@sveltejs/adapter-static`) with no server runtime of
  its own — meant for Cloudflare Pages/Workers, or any static host.
- **Backend** (`backend/`) — Rust, [Axum](https://github.com/tokio-rs/axum) +
  [sqlx](https://github.com/launchbadge/sqlx) against Postgres. Owns all
  persistence and serves `/api/**` plus the `/auth/callback` magic-link
  redirect. Meant to be self-hosted (e.g. a home server/NAS) behind a
  [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/),
  so the frontend's static host and the backend never need to share a
  network — only a public hostname the frontend's `/api` calls are proxied to.

The frontend never talks to Postgres directly and has no server-side code of
its own beyond static file serving — every read/write goes through the Rust
API.

## Running your own copy

### Requirements

- Docker + Docker Compose (for Postgres + the backend)
- Node.js 22+ and npm (for the frontend)
- Rust (only if you want to run the backend outside Docker — see below)

### Setup

```sh
git clone https://github.com/<you>/loosedays.git
cd loosedays
npm install
cp .env.example .env
```

Edit `.env`:

- `SERVER_PEPPER` — a random secret used to hash phone numbers before they're
  ever stored (`openssl rand -hex 32` works well). Changing it later
  invalidates existing phone-number matches.
- `ADMIN_EMAIL` — your own email address, the account the invite-only
  allowlist is bootstrapped with. Everyone else gets invited from the app's
  onboarding screen once you're signed in.

### Run the backend + Postgres

```sh
docker compose up postgres backend
```

This builds the Rust backend, runs the Postgres migrations automatically on
startup, and publishes `:8080` to the host so the frontend dev server can
reach it directly.

Set `PUBLIC_ORIGIN=http://localhost:5173` in `.env` for this flow (matching
Vite's default dev port) — the backend uses it to build the magic-link URL
it emails/prints, and that has to match whatever origin you actually open in
your browser.

### Run the frontend

```sh
npm run dev
```

Open the printed local URL, enter your `ADMIN_EMAIL` on the sign-in screen,
and check the backend's logs — the magic link is printed to the console by
default so you can test without setting up an email provider. Open the link
to sign in.

### Running the whole stack in Docker

`docker compose up` (no service names) builds and runs everything — Postgres,
the Rust backend, and the frontend built as a static bundle served by Caddy,
which also reverse-proxies `/api/**` and `/auth/**` to the backend so the
browser only ever talks to one origin (`http://localhost:8787`). This is the
closest local approximation of the production shape: one public
origin in front, backend split out behind it.

### Sending real email (optional)

By default, magic links are just printed to the backend's logs. To actually
email them (so friends who aren't watching your terminal can sign in), set
these in `.env`:

```
RESEND_API_KEY=your_resend_api_key
EMAIL_FROM=Loose Days <you@yourdomain.com>
```

[Resend](https://resend.com) has a free tier that's plenty for a friend
group. No other provider is wired up out of the box, but
`backend/src/email.rs` is a small, self-contained module if you want to swap
in your own.

### The database

Postgres is provisioned by `docker-compose.yml` with a named volume
(`pgdata`), so data survives container restarts. Back up that volume (or run
`pg_dump` against it) like you would any Postgres instance — there's no
separate migration step to run by hand; the backend applies migrations
(`backend/migrations/`) on startup.

### Deploying

**Backend:** build the Docker image (`backend/Dockerfile`) and run it
wherever you're self-hosting, pointed at a Postgres instance, with
`SERVER_PEPPER`, `ADMIN_EMAIL`, `DATABASE_URL`, `PUBLIC_ORIGIN`, and
optionally `RESEND_API_KEY`/`EMAIL_FROM` set. Put it behind a Cloudflare
Tunnel (or any reverse proxy with TLS) rather than exposing it directly.

**Frontend:** `npm run build` produces a static bundle in `build/` — deploy
it to Cloudflare Pages, Cloudflare Workers (static assets), or any static
host. `static/_redirects` is already set up for SPA-style fallback routing.
Whatever serves the frontend needs to route `/api/**` and `/auth/**` to the
backend's public hostname (same-origin proxying is recommended so session
cookies stay simple — see `Caddyfile` for the pattern used in local dev).

## Development

```sh
npm run dev            # frontend dev server (proxies /api, /auth to :8080)
npm run check          # typecheck
npm run lint           # prettier + eslint
npm run format         # auto-format

cd backend
cargo run              # backend dev server
cargo build --release  # production build
```

## License

MIT — see [LICENSE](./LICENSE).
