# Reedstreams Backend

Rust + Axum + PostgreSQL

## Local Dev

```bash
# Setup postgres locally or use docker
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=password -e POSTGRES_DB=reedstreams postgres:15

# Copy env
cp .env.example .env
# Edit .env with your DB url

# Run migrations and start
cargo run
```

## Deploy to Fly.io

```bash
# Install flyctl if not done
curl -L https://fly.io/install.sh | sh

# Login
fly auth login

# Create app (first time)
fly apps create reedstreams-backend

# Create postgres
fly postgres create --name reedstreams-db

# Attach db to app
fly postgres attach reedstreams-db --app reedstreams-backend

# Set JWT secret
fly secrets set JWT_SECRET="your-secret-key-here"

# Deploy
fly deploy
```

## API Endpoints

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| POST | /auth/register | No | Create account |
| POST | /auth/login | No | Login |
| GET | /profile | Yes | Get my profile |
| PUT | /profile | Yes | Update profile |
| GET | /profile/:id | No | Get any profile |
| GET | /playlists | Yes | List my playlists |
| POST | /playlists | Yes | Create playlist |
| GET | /playlists/:id | Yes | Get playlist |
| PUT | /playlists/:id | Yes | Update playlist |
| DELETE | /playlists/:id | Yes | Delete playlist |
