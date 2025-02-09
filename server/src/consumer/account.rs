use crate::constants::APPLICATION_LIST_REDIS_KEY;
use crate::UserRepository;
use anyhow::{Context, Error};
use eventstore::Client as EventstoreClient;
use hfb_auth_shared::account::AccountEvent;
use hfb_auth_shared::application::{ApplicationEvent, ApplicationList, PrivateApplicationEvent};
use hfb_auth_shared::user::{UserCommand, UserState};
use hfb_auth_shared::{AUTH_ACCOUNT_STREAM, AUTH_APPLICATION_STREAM};
use horfimbor_eventsource::helper::{create_subscription, get_persistent_subscription};
use horfimbor_eventsource::metadata::Metadata;
use horfimbor_eventsource::model_key::ModelKey;
use horfimbor_eventsource::{EventSourceStateError, Stream};
use public_account_event::PubAccountEvent;
use redis::{Client as RedisClient, Commands};
use rocket::tokio::time::MissedTickBehavior::Delay;
use serde_json::json;

pub async fn listen_accounts(
    event_db: &EventstoreClient,
    redis: &RedisClient,
    user_repository: &UserRepository,
) -> Result<(), Error> {
    let stream = Stream::Stream(AUTH_ACCOUNT_STREAM);
    let group_name = "wip";

    create_subscription(event_db, &stream, group_name)
        .await
        .context("cannot create subscription")?;

    let mut sub = get_persistent_subscription(event_db, &stream, group_name)
        .await
        .context("cannot get subscription")?;

    loop {
        let rcv_event = sub.next().await.expect("cannot get next event");

        let full_event = match rcv_event.event.as_ref() {
            None => {
                continue;
            }
            Some(event) => event,
        };

        // FIXME change this metadata check
        let metadata: Metadata = serde_json::from_slice(full_event.custom_metadata.as_ref())
            .context("cannot deserialize")?;

        if !metadata.is_event() {
            sub.ack(rcv_event)
                .await
                .context("cannot acknowledge event")?;

            continue;
        }

        let event = full_event
            .as_json::<AccountEvent>()
            .expect("cannot deserialize");

        match event {
            AccountEvent::Private(_) => {}
            AccountEvent::Public(public) => match public {
                PubAccountEvent::AccountCreated {
                    app_id,
                    user_id,
                    name,
                } => {
                    let account = ModelKey::try_from(full_event.stream_id.as_str())?;

                    match user_repository
                        .add_command(
                            &user_id,
                            UserCommand::AddAccount {
                                application: app_id,
                                account,
                                label: name,
                            },
                            Some(&metadata),
                        )
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            println!("cannot add account :  {e}")
                        }
                    }
                }

                PubAccountEvent::AccountSuspended => {}
                PubAccountEvent::AccountResumed => {}
            },
        }

        sub.ack(rcv_event)
            .await
            .context("cannot acknowledge event")?;
    }

    Ok(())
}
