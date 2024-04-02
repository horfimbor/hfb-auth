use uuid::Uuid;

use anyhow::{anyhow, Result};
use sqlx::{MySql, Pool};

pub struct User {
    uuid: String,
    pseudo: String,
    role: Option<String>,
}

impl User {
    pub fn uuid(&self) -> Uuid {
        self.uuid.parse().unwrap()
    }
}

pub struct MariadDb {
    pub db: Pool<MySql>,
}

impl MariadDb {
    pub fn new(db: Pool<MySql>) -> MariadDb {
        MariadDb { db }
    }

    pub async fn get_user(&self, pseudo: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT uuid, pseudo, role FROM account WHERE pseudo = ?",
            pseudo
        )
        .fetch_all(&self.db)
        .await?;

        Ok(user.into_iter().next())
    }

    pub async fn create_user(&self, pseudo: &str) -> Result<User> {
        let _ = sqlx::query!(
            "insert into account (uuid, pseudo, role) values (uuid(), ?, null)
        ",
            pseudo
        )
        .execute(&self.db)
        .await?;

        self.get_user(pseudo)
            .await?
            .ok_or(anyhow!("user created not found"))
    }
}
