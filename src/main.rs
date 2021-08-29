use std::{time::{Duration, SystemTime}};

use client::http_client::{self, HyperNativeTls};
use hyper::{Error, client::HttpConnector};
use hyper_tls::HttpsConnector;
use ruma::{api::client::r0::{message::send_message_event}, client, events::{AnyMessageEventContent, AnySyncMessageEvent, AnySyncRoomEvent, room::message::{MessageEventContent, MessageType}}, room_id};
use ruma::presence::PresenceState;
use serde_json::Value;
use tokio_stream::StreamExt as _;
use std::io::Read;


#[tokio::main]
async fn main() {
    run().await;
}

type MatrixClient = client::Client<http_client::HyperNativeTls>;
async fn run() {
    // let homeserver_url = "https://test-matrix.iinuwa.xyz".parse().unwrap();
    let homeserver_url = "http://localhost:8080".parse().unwrap();
    let access_token = "VtYOW5H4nQTTv8VFLeDNpAiUwxCKVUrICDOpWb1OHXTh6BT2ItarE0zWdT9hYopdqUiuKMgAiyV5J7047XC1BmcSVsL20rDDdcetQl6UESVsWFww1ED0b03BZWhF9jBG0REy3S4CFYtmfREC9emwHdLOoQoprNporqDdBux4EX9jNmmkqhKQHdNT1t3A7mYSRiEyRd574rPAPrA7BGmMjkZNTVYyXhWXlJzl1LXxQ0wY4hmdQG2UgrwvWTCbdvqE";
    // let device_id = "K2z1kARHNW";
    let client = MatrixClient::new(homeserver_url, Some(access_token.to_owned()));

    /* 
    let session = client
        .log_in("@misterbot:test-matrix.iinuwa.xyz", "misterbot", None, None)
        .await.unwrap();
        */
    
        /* 
    let room_alias = RoomAliasId::try_from( "#bottesting:test-matrix.iinuwa.xyz").unwrap();
    let request1 = get_alias::Request::new(&room_alias);
    let response1 = client
        .send_request(request1)
        .await
        .unwrap();
        */

    let room_id = room_id!("!yoIQlrFtM0kBC9NaDD:test-matrix.iinuwa.xyz");
    // let room_id = get_room_id(&client, &room_alias).await;
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain("hello"));
    let request = send_message_event::Request::new(&room_id, "2", &content);
    let response = client
        .send_request(request)
        .await
        .unwrap();
    println!("{}", response.event_id);

    let next_batch_token = String::from("21607");
    let mut sync_stream = Box::pin(client.sync(
        None,
        next_batch_token,
        &PresenceState::Online,
        Some(Duration::from_secs(30)),
    ));
    let joke_client = hyper::Client::builder().build::<_, hyper::Body>(hyper_tls::HttpsConnector::new());
    println!("Listening...");
    while let Some(response) = sync_stream.try_next().await.unwrap() {
        println!("{}", response.next_batch);
        if let Some(room_info) = response.rooms.join.get(&room_id){
            for e in &room_info.timeline.events {
                if let AnySyncRoomEvent::Message(AnySyncMessageEvent::RoomMessage(m)) = e.deserialize().unwrap() {
                        if let MessageType::Text(t) = m.content.msgtype {
                            println!("{}", t.body);
                            if t.body.contains("joke") {
                                let joke = get_joke(joke_client.clone()).await.unwrap();
                                let joke_content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(joke));
                                let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
                                let req = send_message_event::Request::new(&room_id, &timestamp, &joke_content);
                                client
                                    .send_request(req)
                                    .await
                                    .unwrap();
                            }
                        }
                }
            }
        }
    }
}

async fn get_joke(client: hyper::Client<HttpsConnector<HttpConnector>>) -> Result<String, Error> {
    let uri = "https://v2.jokeapi.dev/joke/Programming?safe-mode&type=single".parse::<hyper::Uri>().unwrap();
    let rsp = client.get(uri).await?;
    let bytes = hyper::body::to_bytes(rsp).await?;
    let json = String::from_utf8(bytes.to_vec()).unwrap();
    let joke_obj = serde_json::from_str::<Value>(&json).unwrap();
    let joke = joke_obj["joke"].as_str().unwrap();
    Ok(joke.to_owned())
}       