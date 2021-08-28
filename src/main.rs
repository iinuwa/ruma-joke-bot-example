use std::{time::Duration};

use client::http_client;
use ruma::{api::client::r0::{message::send_message_event}, client, events::{AnyMessageEventContent, AnySyncMessageEvent, AnySyncRoomEvent, room::message::{MessageEventContent, MessageType}}, room_id};
use ruma::presence::PresenceState;
use tokio_stream::StreamExt as _;


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
    let request = send_message_event::Request::new(&room_id, "1", &content);
    let response = client
        .send_request(request)
        .await
        .unwrap();
    println!("{}", response.event_id);

    let next_batch_token = String::new();
    let mut sync_stream = Box::pin(client.sync(
        None,
        next_batch_token,
        &PresenceState::Online,
        Some(Duration::from_secs(30)),
    ));
    while let Some(response) = sync_stream.try_next().await.unwrap() {
        if let Some(room_info) = response.rooms.join.get(&room_id){
            for e in &room_info.timeline.events {
                if let AnySyncRoomEvent::Message(AnySyncMessageEvent::RoomMessage(m)) = e.deserialize().unwrap() {
                        if let MessageType::Text(t) = m.content.msgtype {
                            println!("{}", t.body)
                        }
                }
            }
        }
    }
}