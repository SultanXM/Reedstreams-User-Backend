# ReedStreams Back-End

ReedStreams is a robust, high-performance API built with Rust using the [Axum](https://github.com/tokio-rs/axum) web framework and [SQLx](https://github.com/launchbadge/sqlx) for asynchronous PostgreSQL interactions. It powers the streaming and community features of the ReedStreams platform.

## 🚀 Key Features

- **User Management**: Secure registration, login (JWT-based), and password resets.
- **Profiles**: Customizable user profiles with avatars, tags, custom colors, and "name glow" effects.
- **Playlists**: User-created playlists for tracking and sharing stream matches.
- **Real-time Chat**: Integrated chat system for community interaction.
- **View Tracking**: Real-time viewer counting using WebSockets and in-memory state.
- **Admin Dashboard**: Specialized routes for managing users and platform settings.
- **Automated Migrations**: Self-healing database schema that updates on startup.

## 🛠️ Tech Stack

- **Language**: Rust 2021 Edition
- **Web Framework**: Axum (0.7)
- **Database**: PostgreSQL with SQLx (async)
- **Authentication**: JWT (jsonwebtoken) & Bcrypt for hashing
- **Real-time**: WebSockets with Tokio broadcast channels
- **Concurrency**: `dashmap` for high-performance in-memory state

---

## 🌍 Deployment to Vercel

This project is configured to be "stand-alone" and requires minimal configuration to get started.

### 1. Environment Variables
Vercel needs these variables set in the **Project Settings > Environment Variables** dashboard:

| Variable | Description | Default (Local) |
| :--- | :--- | :--- |
| `DATABASE_URL` | Your PostgreSQL connection string | `postgres://...` |
| `JWT_SECRET` | Secret key for signing tokens | `dev-secret-...` |

### 2. Deployment Steps
1. Push this repository to GitHub/GitLab/Bitbucket.
2. Import the project into Vercel.
3. Vercel will automatically detect the `vercel.json` and use the `@vercel/rust` builder.
4. Add your `DATABASE_URL` in the Vercel dashboard.
5. Deploy!

> **⚠️ Note on WebSockets:** Vercel Serverless Functions have a timeout and do not support long-lived WebSocket connections natively. While the API will work, the real-time viewer counts via `/ws/views/` may disconnect after the function timeout. For perfect WebSocket support, consider using a persistent host like Fly.io or DigitalOcean.

---

## 💻 Local Development

1. **Clone the repo:**
   ```bash
   git clone <repo-url>
   cd Back-End
   ```

2. **Setup Database:**
   Ensure you have PostgreSQL running and create a database named `reedstreams`.

3. **Run the app:**
   ```bash
   cargo run
   ```
   The server will start at `http://localhost:8080`.

## 🔒 Default Admin Credentials
On the first run, the system automatically creates an admin account:
- **Username**: `admin`
- **Password**: `ReedStreamsAdmin{{0}}` (Change this immediately!)
