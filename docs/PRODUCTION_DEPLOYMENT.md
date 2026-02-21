# Production Deployment — Zorbs

### 1. Prerequisites
- A server / VPS / cloud platform that supports Docker (Railway, Render, Fly.io, DigitalOcean, etc.)
- A domain (zorbs.io) pointed at your server

### 2. Required environment variables (set in your hosting platform)
```
POSTGRES_PASSWORD=your_strong_password
DATABASE_URL=postgres://zorbs:your_strong_password@your-db-host:5432/zorbs
BIND_ADDR=0.0.0.0:3000
UPLOAD_DIR=uploads
```

### 3. Deploy with Docker (recommended)
Use the existing `docker-compose.yml` and `Dockerfile` (already production-ready).

```
docker compose up -d --build
```

### 4. Persistent storage
Make sure `/uploads` is on a persistent volume (Docker volume or mounted host path).

### 5. Reverse proxy (optional but recommended)
Use Nginx / Caddy / Traefik to:
- Handle HTTPS
- Point `zorbs.io` → your container on port 3000

### 6. One-command production start (example for Railway / Render / Fly)
Just push the repo with the Dockerfile — they will auto-build and run.
