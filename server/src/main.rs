#![allow(unused)]

mod constants;
mod consumer;
mod session;
mod url_parsing;
mod user;
mod web;

#[macro_use]
extern crate rocket;
extern crate core;

use crate::consumer::account::listen_accounts;
use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use consumer::application;
use futures::future::try_join_all;
use futures::{FutureExt, StreamExt};
use hfb_auth_shared::account::AccountState;
use hfb_auth_shared::application::ApplicationState;
use hfb_auth_shared::user::{UserCommand, UserRole, UserState};
use hfb_auth_shared::AUTH_USER_STREAM;
use horfimbor_eventsource::cache_db::redis::StateDb;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::repository::{Repository, StateRepository};
use jsonwebtoken::{DecodingKey, EncodingKey};
use kurrentdb::Client as EventstoreClient;
use redis::{Client as RedisClient, Commands};
use ring::signature::{Ed25519KeyPair, KeyPair};
use rocket::tokio::spawn;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::env;
use std::future::Future;
use std::sync::mpsc::{channel, Receiver};
use uuid::Uuid;
use web::public::helper::hash_password;

#[derive(Debug, PartialEq, Clone, ValueEnum)]
enum Service {
    Web,
    Application,
    Account,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    real_env: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    UserUpdate {
        #[arg(short, long)]
        user: Uuid,

        #[arg(long)]
        role: Option<UserRole>,

        #[arg(long)]
        password: Option<String>,
    },
    Service {
        #[arg(long)]
        list: Vec<Service>,
    },
}

type UserStateCache = StateDb<UserState>;
type UserRepository = StateRepository<UserState, UserStateCache>;

type ApplicationStateCache = StateDb<ApplicationState>;
type ApplicationRepository = StateRepository<ApplicationState, ApplicationStateCache>;

type AccountStateCache = StateDb<AccountState>;
type AccountRepository = StateRepository<AccountState, AccountStateCache>;

#[rocket::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();

    if !args.real_env {
        dotenvy::dotenv().context("cannot get env")?;
    }

    let eventstore_uri = env::var("EVENTSTORE_URI")
        .context("fail to get EVENTSTORE_URI env var")?
        .parse()
        .context("fail to parse the settings")?;

    let redis_client =
        RedisClient::open(env::var("REDIS_URI").context("fail to get REDIS_URI env var")?)?;

    let event_store_db = EventstoreClient::new(eventstore_uri)
        .map_err(|e| anyhow!(" cannot connect to eventstore : {e}"))?;

    let auth_user_repository = UserRepository::new(
        event_store_db.clone(),
        UserStateCache::new(redis_client.clone()),
    );
    let application_repository = ApplicationRepository::new(
        event_store_db.clone(),
        ApplicationStateCache::new(redis_client.clone()),
    );
    let account_repository = AccountRepository::new(
        event_store_db.clone(),
        AccountStateCache::new(redis_client.clone()),
    );

    match args.command {
        Command::UserUpdate {
            user,
            password,
            role,
        } => {
            let key = ModelKey::new(AUTH_USER_STREAM, user);

            if let Some(role) = role {
                let admin = auth_user_repository
                    .add_command(&key, UserCommand::ChangeRole(Some(role)), None)
                    .await
                    .context("cannot change role")?;
                dbg!(&admin);
            }

            if let Some(password) = password {
                let admin = auth_user_repository
                    .add_command(
                        &key,
                        UserCommand::ChangePassword {
                            password_hash: hash_password(&password)
                                .context("cannot hash password")?,
                        },
                        None,
                    )
                    .await
                    .context("cannot reset password")?;
                dbg!(&admin);
            }
            Ok(())
        }
        Command::Service { list } => {
            let mut services = vec![];

            if list.is_empty() || list.contains(&Service::Application) {
                services
                    .push(application::listen_applications(&event_store_db, &redis_client).boxed());
            }
            if list.is_empty() || list.contains(&Service::Account) {
                services.push(
                    listen_accounts(&event_store_db, &redis_client, &auth_user_repository).boxed(),
                );
            }

            if list.is_empty() || list.contains(&Service::Web) {
                services.push(
                    web::start_server(
                        &auth_user_repository,
                        &application_repository,
                        &account_repository,
                        &redis_client,
                    )
                    .boxed(),
                );
            }
            let signals = Signals::new([SIGTERM, SIGINT, SIGQUIT])?;
            let handle = signals.handle();

            let signals_task = handle_signals(signals).boxed();
            services.push(signals_task);
            dbg!(services.len());

            try_join_all(services)
                .await
                .map(|_| ())
                .context("some service failed")
        }
    }
}

async fn handle_signals(mut signals: Signals) -> Result<()> {
    if signals.next().await.is_some() {
        bail!("Exit required")
    }

    Ok(())
}
