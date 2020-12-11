use std::error::Error;

use futures::prelude::*;
use serde::{Deserialize, Serialize};
use warp::ws::Message;
use warp::Filter;

use media_server::sdp::webrtc::Session;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum C2SMessage {
    Offer {
        #[serde(with = "serde_with::rust::display_fromstr")]
        sdp: Session,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
enum S2CMessage {
    Answer {
        #[serde(with = "serde_with::rust::display_fromstr")]
        sdp: Session,
    },
}

async fn send_message(websocket: &mut warp::ws::WebSocket, message: &S2CMessage) -> Result<(), Box<dyn Error>> {
    let message = serde_json::to_string(message).unwrap();

    log::info!("sending: {}", message);

    websocket.send(Message::text(message)).await?;

    Ok(())
}

async fn handle_offer(websocket: &mut warp::ws::WebSocket, offer: &Session) -> Result<(), Box<dyn Error>> {
    // TODO: We want to implement something along the lines of the
    //       media-server-node manual signalling example in here.
    //       https://github.com/medooze/media-server-node/blob/master/manual.md

    // let answer = offer.answer();

    // TODO: Just ping back the offer for now.
    //       Can't even do that yet, needs Session::to_sdp implemented.
    // let answer = Clone::clone(offer);

    // send_message(websocket, &S2CMessage::Answer { sdp: answer }).await?;

    Ok(())
}

async fn on_websocket_upgrade(mut websocket: warp::ws::WebSocket) {
    while let Ok(Some(message)) = websocket.try_next().await {
        if message.is_close() {
            log::info!("client closed websocket");
            let _ = websocket.close().await;
            return;
        }

        let text = match message.to_str() {
            Ok(text) => text,
            Err(()) => {
                log::warn!("unexpected message type in websocket: {:?}", message);
                return;
            }
        };

        log::info!("message: {}", text);

        let parsed: C2SMessage = match serde_json::from_str(text) {
            Ok(parsed) => parsed,
            Err(e) => {
                log::warn!("failed to parse websocket message: {}", e);
                return;
            }
        };

        log::info!("parsed: {:?}", parsed);

        match parsed {
            C2SMessage::Offer { sdp } => {
                if let Err(e) = handle_offer(&mut websocket, &sdp).await {
                    log::warn!("failed to handle offer: {}", e);
                    return;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let websocket = warp::get()
        .and(warp::path::path("ws"))
        .and(warp::path::end())
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(on_websocket_upgrade));

    let index = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::html(include_str!("../resources/index.html")));

    let adapter = warp::get()
        .and(warp::path::path("adapter.js"))
        .and(warp::path::end())
        .map(|| {
            let adapter = include_str!("../resources/adapter.js");
            warp::reply::with_header(adapter, "Content-Type", "application/javascript")
        });

    let favicon = warp::get()
        .and(warp::path::path("favicon.ico"))
        .and(warp::path::end())
        .map(|| {
            let favicon = include_bytes!("../resources/favicon.ico");
            warp::reply::with_header(favicon.as_ref(), "Content-Type", "image/vnd.microsoft.icon")
        });

    let routes = websocket.or(index).or(adapter).or(favicon).with(warp::log("sfu::http"));

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
