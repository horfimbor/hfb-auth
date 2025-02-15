use crate::constants::APPLICATION_LIST_REDIS_KEY;
use anyhow::{Context, Result};
use eventstore::Client as EventstoreClient;
use hfb_auth_shared::application::{ApplicationEvent, ApplicationList, PrivateApplicationEvent};
use hfb_auth_shared::AUTH_APPLICATION_STREAM;
use horfimbor_eventsource::helper::{create_subscription, get_persistent_subscription};
use horfimbor_eventsource::metadata::Metadata;
use horfimbor_eventsource::Stream;
use redis::{Client as RedisClient, Commands};
use serde_json::json;

pub async fn listen_applications(event_db: &EventstoreClient, redis: &RedisClient) -> Result<()> {
    let stream = Stream::Stream(AUTH_APPLICATION_STREAM);
    let group_name = "oups";

    create_subscription(event_db, &stream, group_name)
        .await
        .context("cannot create subscription")?;

    let mut sub = get_persistent_subscription(event_db, &stream, group_name)
        .await
        .context("cannot get subscription")?;

    let mut connection = redis.get_connection().context("cannot connect to redis")?;

    let raw_data: Option<String> = connection
        .get(APPLICATION_LIST_REDIS_KEY)
        .context("cannot get data")?;

    let mut application_list: Vec<ApplicationList> = match raw_data {
        None => Vec::new(),
        Some(list) => {
            serde_json::from_str(&list).context("cannot deserialize application list in redis")?
        }
    };
    loop {
        let rcv_event = sub.next().await.context("cannot get next event")?;

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
            .as_json::<ApplicationEvent>()
            .context("cannot deserialize")?;

        match event {
            ApplicationEvent::Private(prv) => match prv {
                PrivateApplicationEvent::Created { name, host, key } => {
                    application_list.push(ApplicationList {
                        id: full_event.id.to_string(),
                        name,
                        host,
                    });

                    let data = json!(application_list.clone()).to_string();

                    connection
                        .set::<_, _, ()>(APPLICATION_LIST_REDIS_KEY, data)
                        .context("cannot set data in redis")?;
                }
                PrivateApplicationEvent::KeyChanged { .. } => {}
            },
        }

        sub.ack(rcv_event)
            .await
            .context("cannot acknowledge event")?;
    }

    Ok(())
}
