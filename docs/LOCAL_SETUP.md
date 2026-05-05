# Local Development Setup — Zorbs

Follow these steps **every time** you want to run the registry locally.

### 1. Clone & enter the repo
```
git clone https://github.com/murphsicles/zorbs.git
cd zorbs
```

### 2. Create .env (never commit this)
```
cp .env.example .env
```
Edit `.env` and make sure it contains:
```
POSTGRES_PASSWORD=zorbs_dev
DATABASE_URL=postgres://zorbs:zorbs_dev@localhost:5432/zorbs
BIND_ADDR=0.0.0.0:3000
UPLOAD_DIR=uploads
```

### 3. Start the database
```
docker compose up -d db
```

### 4. Prepare sqlx cache (required for build)
```
$env:DATABASE_URL = "postgres://zorbs:zorbs_dev@localhost:5432/zorbs"
cargo sqlx prepare --workspace
$env:SQLX_OFFLINE = "true"
```

### 5. Run the server
**Option A (recommended for dev):**
```
cargo run --bin zorbs
```

**Option B (full Docker stack):**
```
docker compose up --build
```

### 6. Open the site
http://localhost:3000

### 7. Stop everything
- Ctrl + C (for `cargo run`)
- `docker compose down` (for Docker)
