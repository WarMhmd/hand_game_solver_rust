# Hand Game Solver (Rust + React)

Hosted: https://hand.warmhmd.online/

This repo contains:
- `Backend/`: Rust (Axum + Tokio) HTTP API + WebSocket server
- `frontend/`: React + TypeScript (Vite) web UI

## Backend

### Endpoints (verified in code)

Backend listens on `0.0.0.0:3000`.

- `GET /`  returns a simple "Backend is running" string
- `GET /health` returns `OK`
- WebSocket: `GET /ws`

Game APIs:
- `POST /api/game/init_game`
- `POST /api/game/init_game_with_random_strong_bots`
- `POST /api/game/init_game_with_random_all_bots`

Bot APIs:
- `POST /api/v1/bot/init-bot`
- `POST /api/v1/bot/draw-card`
- `POST /api/v1/bot/meld-cards`
- `POST /api/v1/bot/play-in-melds`
- `POST /api/v1/bot/discard`

### Run locally

From the repo root:

```bash
cd Backend
cargo run
```

### CORS

The backend CORS allowlist includes `http://localhost:3001` and `https://hand.warmhmd.online`.

## Frontend

### Run locally

```bash
cd frontend
npm install
npm run dev
```

The dev server runs on `http://localhost:3001`.

### Configure backend URLs (optional)

The frontend defaults to local endpoints, but can be overridden with Vite env vars:

- `VITE_GAME_API_ENDPOINT` (default: `http://localhost:3000/api/game`)
- `VITE_WS_ENDPOINT` (default: `ws://localhost:3000/ws`)

## Docker

### Backend image

```bash
cd Backend
docker build -t hand-game-backend .
docker run -p 3000:3000 hand-game-backend
```

### Docker Compose

`docker-compose.yml` currently defines only the backend service and uses `expose: 3000` (intended for use behind a reverse proxy like Traefik).

If you want to access it directly on your machine without a reverse proxy, add a port mapping under `backend:`:

```yaml
ports:
  - "3000:3000"
```
