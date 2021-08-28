use std::convert::TryFrom;

use client::http_client;
use ruma::{RoomAliasId, RoomId, api::client::r0::{alias::get_alias, message::send_message_event}, client, events::{AnyMessageEventContent, room::message::MessageEventContent}, identifiers::_macros::room_alias_id, room_id};


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
}

/*
async fn get_room_id(client: &MatrixClient, room_alias: &RoomAliasId) -> RoomId {
}
*/
