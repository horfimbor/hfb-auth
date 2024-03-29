use uuid::Uuid;

use anyhow::Result;
use sqlx::{MySql, Pool};

pub struct User {
    uuid: Uuid,
    pseudo: String,
    role: String,
}

pub struct MariadDb {
    pub db: Pool<MySql>,
}

impl MariadDb {
    pub fn new(db: Pool<MySql>) -> MariadDb {
        MariadDb { db }
    }

    pub async fn get_user(&self, pseudo: &str) -> Result<User> {
        let users = sqlx::query!(
            "
SELECT uuid, pseudo, role
FROM account
WHERE pseudo = ?
        ",
            pseudo
        )
        .fetch_all(&self.db) // -> Vec<{ country: String, count: i64 }>
        .await?;

        todo!("WIP !")
    }
}
