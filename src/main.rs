use std::{
    convert::TryInto,
    fs,
    io,
    error::Error,
    time::{Duration, SystemTime},
};

use client::http_client;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use ruma::{
    api::client::r0::{filter::FilterDefinition, message::send_message_event, sync::sync_events},
    assign, client,
    events::{
        room::message::{MessageEventContent, MessageType},
        AnyMessageEventContent, AnySyncMessageEvent, AnySyncRoomEvent,
    },
    UserId,
};

use ruma::presence::PresenceState;
use serde_json::Value;
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() {
    run().await;
}

type MatrixClient = client::Client<http_client::HyperNativeTls>;
async fn run() {
    let config = read_config().expect("valid configuration in ./config");
    let http_client =
        hyper::Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
    let client = if let Some(state) = read_state().ok().flatten() {
        MatrixClient::with_http_client(http_client.clone(), config.homeserver.to_owned(), Some(state.access_token))
    } else if let Some(password) = &config.password {
        let client = MatrixClient::with_http_client(http_client.clone(), config.homeserver.to_owned(), None);
        match
        client
            .log_in(config.username.as_ref(), password, None, None)
            .await {
                Ok(_) => client,
                Err(e) => {
                    let reason = match e {
                        client::Error::AuthenticationRequired => "invalid credentials specified".to_string(),
                        client::Error::Response(response_err) => format!("failed to get a response from the server: {}", response_err),
                        client::Error::FromHttpResponse(parse_err) => format!("failed to parse log in response: {}", parse_err),
                        _ => e.to_string(),
                    };
                    panic!("Failed to log in: {}", reason);
                }
            }
    } else {
        panic!("No previous session found and no credentials stored in config")
    };

    let filter = FilterDefinition::ignore_all().into();
    let initial_sync_response = client
        .send_request(assign!(sync_events::Request::new(), {
            filter: Some(&filter),
        }))
        .await
        .unwrap();
    let user_id = &config.username;
    let not_senders = &[user_id.clone()];
    let filter = {
        let mut filter = FilterDefinition::empty();
        filter.room.timeline.not_senders = not_senders;
        filter
    }
    .into();

    let mut sync_stream = Box::pin(client.sync(
        Some(&filter),
        initial_sync_response.next_batch,
        &PresenceState::Online,
        Some(Duration::from_secs(30)),
    ));
    println!("Listening...");
    while let Some(response) = sync_stream.try_next().await.unwrap() {
        write_state(&State {
            access_token: client.access_token().expect("logged in client"),
        })
        .unwrap();
        println!("{}", response.next_batch);
        for (room_id, room_info) in response.rooms.join {
            for e in &room_info.timeline.events {
                if let AnySyncRoomEvent::Message(AnySyncMessageEvent::RoomMessage(m)) =
                    e.deserialize().unwrap()
                {
                    // workaround because Conduit does not implement filtering.
                    if &m.sender == user_id { continue; }

                    if let MessageType::Text(t) = m.content.msgtype {
                        println!("{}:\t{}", m.sender, t.body);
                        if t.body.to_ascii_lowercase().contains("joke") {
                            let joke = get_joke(&http_client.clone()).await.unwrap();
                            let joke_content = AnyMessageEventContent::RoomMessage(
                                MessageEventContent::text_plain(joke),
                            );
                            let timestamp = SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                                .to_string();
                            let req = send_message_event::Request::new(
                                &room_id,
                                &timestamp,
                                &joke_content,
                            );
                            client.send_request(req).await.unwrap();
                        }
                    }
                }
            }
        }

        for (room_id, _) in response.rooms.invite {
            println!("invited to {}", &room_id);
            client
                .send_request(
                    ruma::api::client::r0::membership::join_room_by_id::Request::new(&room_id),
                )
                .await
                .unwrap();

            let greeting = "Hello! My name is Mr. Bot! I like to tell jokes. Like this one: ";
            let joke = get_joke(&http_client).await.unwrap();
            let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(
                format!("{}\n{}", greeting, joke),
            ));
            client
                .send_request(send_message_event::Request::new(
                    &room_id,
                    &response.next_batch,
                    &content,
                ))
                .await
                .unwrap();
        }
    }
}

async fn get_joke(client: &hyper::Client<HttpsConnector<HttpConnector>>) -> Result<String, Box<dyn Error>> {
    let uri = "https://v2.jokeapi.dev/joke/Programming,Pun,Misc?safe-mode&type=single"
        .parse::<hyper::Uri>()
        .unwrap();
    let rsp = client.get(uri).await?;
    let bytes = hyper::body::to_bytes(rsp).await?;
    let json = String::from_utf8(bytes.to_vec()).unwrap();
    let joke_obj = serde_json::from_str::<Value>(&json).unwrap();
    let joke = joke_obj["joke"].as_str().unwrap();
    Ok(joke.to_owned())
}

struct State {
    access_token: String,
}

fn write_state(state: &State) -> Result<(), std::io::Error> {
    let content = &state.access_token;
    fs::write("./session", content)?;
    Ok(())
}

fn read_state() -> Result<Option<State>, io::Error> {
    match fs::read_to_string("./session") {
        Ok(access_token) => Ok(Some(State { access_token })),
        Err(e) => {
            if let io::ErrorKind::NotFound = e.kind() {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

struct Config {
    homeserver: String,
    username: UserId,
    password: Option<String>,
}

fn read_config() -> Result<Config, io::Error> {
    let content = fs::read_to_string("./config")?;
    let lines = content.split('\n');

    let mut homeserver = None;
    let mut username = None;
    let mut password = None;
    for line in lines {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "homeserver" => homeserver = Some(value.trim().to_owned()),
                // TODO: infer domain from `homeserver`
                "username" => {
                    username = Some(
                        value
                            .trim()
                            .to_owned()
                            .try_into()
                            .expect("Matrix User ID in correct format"),
                    )
                }
                "password" => password = Some(value.trim().to_owned()),
                _ => {}
            }
        }
    }
    if let (Some(homeserver), Some(username)) = (homeserver, username) {
        Ok(Config {
            homeserver,
            username,
            password,
        })
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "`homeserver` and username are required required",
        ))
    }
}
