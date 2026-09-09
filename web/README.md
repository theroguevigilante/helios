# Helios Frontend

The interactive web dashboard for Helios. Built for extreme performance and smooth forensic analysis.

## Tech Stack
- **Framework**: SvelteKit (Svelte 5 Runes)
- **Styling**: Tailwind CSS v4
- **Charts**: Apache ECharts (Canvas-rendered for handling massive datasets)
- **Virtualization**: `@tanstack/svelte-virtual` for infinite scrolling of millions of log events.
- **Build Target**: Fully static SPA export (`@sveltejs/adapter-static`)

## Development

```bash
npm install
npm run dev
```

The frontend will automatically proxy `/api/*` requests to the Rust backend running locally on `http://localhost:8080`.

## Production Build

To generate the static HTML/JS/CSS bundle for deployment:

```bash
npm run build
```
The output will be placed in the `build/` directory, ready to be served by Caddy, Nginx, or Docker.
