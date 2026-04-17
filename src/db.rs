use sqlx::PgPool;
use std::env;
use uuid::Uuid;

pub async fn init_db() -> Result<PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/reedstreams".to_string());

    let pool = PgPool::connect(&database_url).await?;
    
    
    run_migrations(&pool).await?;
    
    
    create_default_admin(&pool).await?;
    
    Ok(pool)
}

async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    
    sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
        .execute(pool)
        .await?;
    
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            username VARCHAR(32) UNIQUE NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            is_admin BOOLEAN NOT NULL DEFAULT FALSE,
            timeout_until TIMESTAMP,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await.ok();
    
    
    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS email VARCHAR(255)")
        .execute(pool)
        .await?;
    
    
    sqlx::query("UPDATE users SET email = username || '@temp.local' WHERE email IS NULL")
        .execute(pool)
        .await?;
    
    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT FALSE")
        .execute(pool)
        .await?;
    
    sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS timeout_until TIMESTAMP")
        .execute(pool)
        .await?;
    
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS profiles (
            user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
            tags TEXT[],
            memes TEXT[],
            theme VARCHAR(50) DEFAULT 'dark',
            avatar_url TEXT,
            profile_pic_url TEXT,
            name_color VARCHAR(7) DEFAULT '#3b82f6',
            description TEXT,
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await.ok();
    
    
    sqlx::query("ALTER TABLE profiles ADD COLUMN IF NOT EXISTS profile_pic_url TEXT")
        .execute(pool)
        .await?;
    
    sqlx::query("ALTER TABLE profiles ADD COLUMN IF NOT EXISTS name_color VARCHAR(7) DEFAULT '#3b82f6'")
        .execute(pool)
        .await?;
    
    sqlx::query("ALTER TABLE profiles ADD COLUMN IF NOT EXISTS description TEXT")
        .execute(pool)
        .await?;
    
    sqlx::query("ALTER TABLE profiles ADD COLUMN IF NOT EXISTS name_glow INTEGER DEFAULT 8")
        .execute(pool)
        .await?;
    
    sqlx::query("ALTER TABLE profiles ADD COLUMN IF NOT EXISTS badge VARCHAR(20)")
        .execute(pool)
        .await?;
    
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS playlists (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            name VARCHAR(255) NOT NULL,
            description TEXT,
            matches TEXT[] NOT NULL DEFAULT '{}',
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;
    
    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS chat_messages (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            username VARCHAR(32) NOT NULL,
            content TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;

    
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS password_resets (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token VARCHAR(255) UNIQUE NOT NULL,
            expires_at TIMESTAMP NOT NULL,
            used BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;
    
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_playlists_user_id ON playlists(user_id)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at ON chat_messages(created_at)")
        .execute(pool)
        .await?;

    // Add match tracking tables
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS matches (
            id VARCHAR(255) PRIMARY KEY,
            views INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;

    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS match_views (
            id SERIAL PRIMARY KEY,
            match_id VARCHAR(255) NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
            ip_address VARCHAR(45) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            UNIQUE(match_id, ip_address)
        )
    "#).execute(pool).await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_match_views_match_id ON match_views(match_id)")
        .execute(pool)
        .await?;

    // Add default source settings table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS default_source_settings (
            id SERIAL PRIMARY KEY,
            source_name VARCHAR(50) UNIQUE NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT FALSE,
            priority INTEGER NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
    "#).execute(pool).await?;

    // Insert actual sources from streamed.pk API
    sqlx::query(r#"
        INSERT INTO default_source_settings (source_name, is_default, priority, is_active)
        VALUES 
            ('echo', TRUE, 1, TRUE),
            ('delta', FALSE, 2, TRUE),
            ('admin', FALSE, 3, TRUE),
            ('golf', FALSE, 4, TRUE)
        ON CONFLICT (source_name) DO NOTHING
    "#).execute(pool).await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_default_source_active ON default_source_settings(is_active, is_default, priority)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn create_default_admin(pool: &PgPool) -> Result<(), sqlx::Error> {
    use bcrypt::{hash, DEFAULT_COST};
    
    
    let admin_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = 'admin')"
    )
    .fetch_one(pool)
    .await?;
    
    if !admin_exists.0 {
        let password_hash = hash("ReedStreamsAdmin{{0}}", DEFAULT_COST)
            .expect("Failed to hash admin password");
        
        let user_id: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_admin)
            VALUES (uuid_generate_v4(), 'admin', 'admin@reedstreams.local', $1, TRUE)
            RETURNING id
            "#
        )
        .bind(&password_hash)
        .fetch_one(pool)
        .await?;
        
        
        sqlx::query(
            "INSERT INTO profiles (user_id, tags, memes, name_color) VALUES ($1, ARRAY['admin'], ARRAY['https://cdn.discordapp.com/emojis/123.png'], '#ff0000')"
        )
        .bind(&user_id.0)
        .execute(pool)
        .await?;
        
        println!(" Default admin created!");
        println!("   Username: admin");
        println!("   Password: ReedStreams@2024!Admin");
    }
    
    Ok(())
}
