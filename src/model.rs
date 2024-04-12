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
    app_key: String,
}

impl Application {
    pub fn uuid(&self) -> &str {
        &self.uuid
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn host(&self) -> &str {
        &self.host
    }
    pub fn app_key(&self) -> &str {
        &self.app_key
    }
}

#[derive(Debug)]
pub struct OneTimeToken {
    application_id: String,
    account_id: String,
    token: String,
}

impl OneTimeToken {
    pub fn application_id(&self) -> &str {
        &self.application_id
    }
    pub fn account_id(&self) -> &str {
        &self.account_id
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

    pub async fn get_application_by_host(&self, host: &str) -> Result<Option<Application>> {
        let application = sqlx::query_as!(
            Application,
            "Select uuid, name, host, app_key from application where host = ?",
            host
        )
        .fetch_all(&self.db)
        .await?;

        dbg!(&application);

        Ok(application.into_iter().next())
    }
    pub async fn get_application(&self, id: &str) -> Result<Option<Application>> {
        let application = sqlx::query_as!(
            Application,
            "Select uuid, name, host, app_key from application where uuid = ?",
            id
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

    pub async fn get_one_time_token(&self, token: &str) -> Result<Option<OneTimeToken>> {
        let token = sqlx::query_as!(
            OneTimeToken,
            "Select application_id, account_id, token from token_one_time where token = ?",
            token
        )
        .fetch_all(&self.db)
        .await?;

        dbg!(&token);

        Ok(token.into_iter().next())
    }
}
