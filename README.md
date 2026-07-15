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
invited from inside the app.

Built with SvelteKit + Tailwind on the frontend, and a small self-hosted
backend: SQLite for storage, session cookies for auth, no external services
required to get running.

## Running your own copy

### Requirements

- Node.js 22+
- npm

### Setup

```sh
git clone https://github.com/<you>/loosedays.git
cd loosedays
npm install
cp .env.example .env
```

Edit `.env` and set `ADMIN_EMAIL` to your own email address — that's the
account the invite-only allowlist is bootstrapped with. Everyone else gets
invited from the app's onboarding screen once you're signed in.

```sh
npm run dev
```

Open the printed local URL, enter your `ADMIN_EMAIL` on the sign-in screen,
and check your terminal — the magic link is logged to the server console by
default so you can test without setting up an email provider. Open the link
to sign in.

### Sending real email (optional)

By default, magic links are just printed to the server logs. To actually
email them (so friends who aren't watching your terminal can sign in), set
these in `.env`:

```
RESEND_API_KEY=your_resend_api_key
EMAIL_FROM=Loose Days <you@yourdomain.com>
```

[Resend](https://resend.com) has a free tier that's plenty for a friend
group. No other provider is wired up out of the box, but `src/lib/server/email.ts`
is a small, self-contained module if you want to swap in your own.

### The database

Data is stored in a single SQLite file (`DATABASE_PATH` in `.env`, defaults to
`./data/loosedays.db`), created automatically on first run. Back it up like
any other file — there's no separate database server to manage.

### Deploying

The app builds to a standalone Node server via
[`@sveltejs/adapter-node`](https://svelte.dev/docs/kit/adapter-node):

```sh
npm run build
node build
```

Set `ADMIN_EMAIL`, `DATABASE_PATH`, and (optionally) `RESEND_API_KEY` /
`EMAIL_FROM` as environment variables wherever you deploy — a small VPS, a
container, whatever you've already got. Make sure `DATABASE_PATH` points
somewhere with persistent storage across restarts/deploys.

## Development

```sh
npm run dev          # start the dev server
npm run check         # typecheck
npm run lint          # prettier + eslint
npm run format         # auto-format
```

## License

MIT — see [LICENSE](./LICENSE).
