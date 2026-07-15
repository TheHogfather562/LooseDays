import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	server: {
		// Lets `npm run dev` talk to a backend started separately (e.g. `cargo
		// run` in backend/, or `docker compose up postgres backend`) without
		// needing the full docker-compose proxy stack running.
		proxy: {
			'/api': 'http://localhost:8080',
			'/auth': 'http://localhost:8080'
		}
	},
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},

			// The whole app renders client-side (ssr = false in the root layout) and
			// talks to the Rust backend over /api/**, so this builds to a static
			// SPA bundle — no Node server needed — for Cloudflare Pages/Workers.
			adapter: adapter({
				fallback: 'index.html'
			})
		})
	]
});
