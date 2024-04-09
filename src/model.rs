use uuid::Uuid;

use anyhow::{anyhow, Result};
use sqlx::{MySql, Pool};

pub struct Account {
    uuid: String,
    pseudo: String,
    role: Option<String>,
}

impl Account {
    pub fn uuid(&self) -> Uuid {
        self.uuid.parse().unwrap()
    }
}

#[derive(Debug)]
pub struct Application {
    uuid: String,
    name: String,
    host: String,
}

impl Application {
    pub fn uuid(&self) -> Uuid {
        self.uuid.parse().unwrap()
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct MariadDb {
    pub db: Pool<MySql>,
}

impl MariadDb {
    pub fn new(db: Pool<MySql>) -> MariadDb {
        MariadDb { db }
    }

    pub async fn get_user(&self, pseudo: &str) -> Result<Option<Account>> {
        let user = sqlx::query_as!(
            Account,
            "SELECT uuid, pseudo, role FROM account WHERE pseudo = ?",
            pseudo
        )
        .fetch_all(&self.db)
        .await?;

        Ok(user.into_iter().next())
    }

    pub async fn get_user_by_id(&self, uuid: &str) -> Result<Option<Account>> {
        let user = sqlx::query_as!(
            Account,
            "SELECT uuid, pseudo, role FROM account WHERE uuid = ?",
            uuid
        )
        .fetch_all(&self.db)
        .await?;

        Ok(user.into_iter().next())
    }

    pub async fn create_user(&self, pseudo: &str) -> Result<Account> {
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

    pub async fn get_application(&self, host: &str) -> Result<Option<Application>> {
        let application = sqlx::query_as!(
            Application,
            "Select uuid, name, host from application where host = ?",
            host
        )
        .fetch_all(&self.db)
        .await?;

        dbg!(&application);

        Ok(application.into_iter().next())
    }

    pub async fn new_one_time_token(
        &self,
        application: &Application,
        account: &Account,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query!(
            "insert into token_one_time (application_id, account_id, token) values (?,?,?) ",
            application.uuid().to_string(),
            account.uuid().to_string(),
            id.to_string()
        )
        .execute(&self.db)
        .await?;

        Ok(id)
    }
}
