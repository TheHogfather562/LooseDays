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
invited from inside the app. Once signed in, a user can also register a
passkey (Account screen) for faster, usernameless sign-in on that device
afterward — magic link remains the way to sign in on a new device or if a
passkey is lost. Phone numbers are only ever stored as a salted hash (used to
match friends against each other), never in the clear.

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
docker compose -f docker-compose.yml -f docker-compose.dev.yml up postgres backend
```

This builds the Rust backend, runs the Postgres migrations automatically on
startup, and (via the dev overlay) publishes `:8080` to the host so the
frontend dev server can reach it directly. The base `docker-compose.yml`
doesn't publish that port on its own — Caddy reaches the backend over the
compose network in the full-stack flow below, so it's only needed for this
split dev setup.

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
it to any static host. `static/_redirects` is already set up for
SPA-style fallback routing. Whatever serves the frontend needs to route
`/api/**` and `/auth/**` to the backend's public hostname (same-origin
proxying is recommended so session cookies stay simple — see `Caddyfile`
for the pattern used in local dev and by `docker-compose.prod.yml` below).

**Or: the whole stack on one box (e.g. a NAS)** — `docker-compose.prod.yml`
runs Postgres, the backend, and the Caddy-served frontend together, with a
`cloudflared` container that tunnels a public hostname to `proxy:80` so
nothing needs a port forwarded on your router:

```sh
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```

Requires `CLOUDFLARE_TUNNEL_TOKEN` in `.env` — create a tunnel with the
Docker connector in the Cloudflare Zero Trust dashboard (Networks ->
Tunnels), point its public hostname route at `http://proxy:80`, and paste
the token in. Set `PUBLIC_ORIGIN`/`COOKIE_SECURE` in `.env` to match that
public hostname (`https://...`, `COOKIE_SECURE=true`) before starting.

### Continuous deployment

`.github/workflows/deploy.yml` redeploys the whole stack — Postgres,
backend, and the Caddy-served frontend — on every push to `main` (skipping
docs-only changes). It runs `docker compose -f docker-compose.yml -f
docker-compose.prod.yml up -d --build` against a **persistent checkout on
the box itself**, via a **self-hosted GitHub Actions runner** installed on
that same machine — so nothing needs to accept inbound connections from
GitHub; the runner polls GitHub outbound, same as the `cloudflared`
container does.

The workflow points at whatever directory already holds your live checkout
(`working-directory:` in `deploy.yml`) — it deliberately does **not** clone
a fresh copy, because that would risk standing up a second stack against an
empty Postgres volume instead of the real `loosedays_pgdata` one. If your
existing checkout lives somewhere other than what's currently configured,
update `working-directory:` in `deploy.yml` to match before going further.
The Compose project name is pinned explicitly (`name: loosedays` in
`docker-compose.yml`), so it always resolves to `loosedays_pgdata`
regardless of which directory or user runs the command.

To set it up on the box (e.g. the Proxmox VM), reusing the existing clone
and its existing OS user rather than adding a new one:

1. Confirm the existing checkout is clean and has no uncommitted local
   edits (`git status` in it) — the workflow runs `git fetch` + `git reset
   --hard origin/main` there on every deploy, so anything uncommitted would
   be silently discarded on the first automated run. `.env` is gitignored
   and untouched by this either way.
2. If the user that owns the checkout isn't already in the `docker` group,
   add it (log out/in, or start a new shell, for the group change to take
   effect):
   ```sh
   sudo usermod -aG docker <user>
   ```
3. GitHub repo -> Settings -> Actions -> Runners -> New self-hosted runner,
   and follow the generated download/config commands **as that same
   user**. When prompted for labels, add `loosedays` (the workflow targets
   `[self-hosted, loosedays]` specifically, so it won't pick up jobs from
   any other self-hosted runner you might register elsewhere).
4. Install it as a service so it survives reboots:
   ```sh
   sudo ./svc.sh install <user>
   sudo ./svc.sh start
   ```

No GitHub repo secrets are needed for this workflow — everything it needs
(`CLOUDFLARE_TUNNEL_TOKEN`, etc.) already lives in `/opt/loosedays/.env` on
the box. It can also be triggered manually from the Actions tab
(`workflow_dispatch`) if you need to redeploy without a new commit.

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
