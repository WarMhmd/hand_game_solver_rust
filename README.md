# Hand Game Solver (Rust + React)

A game/hand-solver project with:
- A Rust backend (Axum) that hosts HTTP APIs + a WebSocket endpoint
- A React + TypeScript (Vite) frontend that talks to the backend

## Repo layout

- `Backend/` — Rust server (Axum + Tokio)
- `frontend/` — React app (Vite + Tailwind)

## Backend

### What it exposes

- HTTP API base: `http://localhost:3000/api/game`
  - `POST /init_game` → creates a new game and returns the initial `GameState`
- WebSocket endpoint: `ws://localhost:3000/ws`

The WebSocket layer uses an acknowledgment system so the server can (optionally) wait for clients to confirm they received certain events before continuing game logic.

### Run locally

```bash
cd Backend
cargo run
```

You should see logs like:
- `Backend running on http://0.0.0.0:3000`
- `WebSocket endpoint: ws://0.0.0.0:3000/ws`

## Frontend

### Run locally

```bash
cd frontend
npm install
npm run dev
```

### Build

```bash
cd frontend
npm run build
```

## Docker Deployment

### Using Docker Compose

```bash
docker-compose up
```

This will start:
- **Backend**: http://localhost:3000
- **Frontend**: http://localhost:3001

### Individual Services

Backend:
```bash
cd Backend
docker build -t hand-game-backend .
docker run -p 3000:3000 hand-game-backend
```

Frontend:
```bash
cd frontend
docker build -t hand-game-frontend .
docker run -p 3001:3001 hand-game-frontend
```

## Tech stack

- Backend: Rust (edition 2021), Axum, Tokio, Serde
- Frontend: React + TypeScript, Vite, Tailwind

## Notes

- The backend binds to `0.0.0.0:3000` for Docker compatibility
- WebSocket message types and the ack/wait architecture are documented in `Backend/src/websocket/README.md`
