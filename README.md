# Hand Game Solver (Rust + React)

Live site: https://hand-solver.web.app/

A game/hand-solver project with:
- A Rust backend (Axum) that hosts HTTP APIs + a WebSocket endpoint
- A React + TypeScript (Vite) frontend that talks to the backend

## Repo layout

- `Backend/` — Rust server (Axum + Tokio)
- `frontend/` — React app (Vite + Tailwind)

## Backend

### What it exposes

- HTTP API base: `http://127.0.0.1:3000/api/game`
  - `POST /init_game` → creates a new game and returns the initial `GameState`
- WebSocket endpoint: `ws://127.0.0.1:3000/ws`

The WebSocket layer uses an acknowledgment system so the server can (optionally) wait for clients to confirm they received certain events before continuing game logic.

### Run locally (Windows / PowerShell)

```powershell
cd Backend
cargo run
```

You should see logs like:
- `Backend running on http://localhost:3000`
- `WebSocket endpoint: ws://localhost:3000/ws`

## Frontend

### Config

The frontend calls the backend API via:

- `VITE_GAME_API_ENDPOINT` (defaults to `http://localhost:3000/api/game`)

If your backend is not running on `localhost:3000`, set `VITE_GAME_API_ENDPOINT` accordingly.

### Run locally (Windows / PowerShell)

```powershell
cd frontend
npm install
npm run dev
```

### Build

```powershell
cd frontend
npm run build
npm run preview
```

## Quick start (both)

In two terminals:

1) Backend
```powershell
cd Backend
cargo run
```

2) Frontend
```powershell
cd frontend
npm install
npm run dev
```

## Tech stack

- Backend: Rust (edition 2021), Axum, Tokio, Serde
- Frontend: React + TypeScript, Vite, Tailwind

## Notes

- The backend currently binds to `127.0.0.1:3000` (see `Backend/src/main.rs`).
- WebSocket message types and the ack/wait architecture are documented in `Backend/src/websocket/README.md`.
