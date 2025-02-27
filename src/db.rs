use serenity::all::User;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use time::PrimitiveDateTime;
use tracing::error;

pub struct Database {
    pool: Pool<Postgres>
}

impl Database {   
    pub async fn init(url: &str) -> Database {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
                .await
                .expect("Failed to connect to database");

        sqlx::migrate!().run(&pool)
            .await
            .expect("Migration failed");

        return Database{pool};
    }

    pub async fn add_user(&self, user: &User, today: PrimitiveDateTime) {
        let id = user.id.to_string();
        let result = sqlx::query!("INSERT INTO discord (id, date) VALUES ($1, $2)", id, today)
            .execute(&self.pool)
            .await;
        if let Err(e) = result {
            error!("Failed to add user: {e}")
        }
    }

    pub async fn remove_user(&self, user: &User) {
        let id = user.id.to_string();
        let result = sqlx::query!("DELETE FROM discord WHERE id = $1", id)
            .execute(&self.pool)
            .await;
        if let Err(e) = result {
            error!("Failed to remove user: {e}")
        }
    }

    pub async fn delete_users(&self, cutoff_date: PrimitiveDateTime) -> Vec<DbUser> {
        let deleted_users: Vec<DbUser> = sqlx::query_as!(DbUser, "DELETE FROM discord WHERE date < $1 RETURNING id, date",
            cutoff_date)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to fetch data from database {e}");
                vec![]
            });

        return deleted_users
    }


}

#[derive(sqlx::FromRow)]
pub struct DbUser {
    pub id: String,
    pub date: PrimitiveDateTime
}