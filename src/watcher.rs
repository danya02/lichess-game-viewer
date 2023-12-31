use futures_util::{SinkExt, StreamExt};
use rand::{thread_rng, Rng};
use serde::Serialize;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{connect, Message},
};

use crate::{
    game_list::{get_game_list, GameCategory},
    types::GameId,
};

pub struct Watcher {
    websocket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    watched_games: Vec<GameId>,
}

impl Watcher {
    pub async fn new() -> Self {
        let mut rng = thread_rng();
        let mut sri = String::new();
        for _ in 0..12 {
            sri.push(rng.sample(rand::distributions::Alphanumeric) as char);
        }
        let (ws_stream, _) =
            connect_async(format!("wss://socket2.lichess.org/socket/v5?sri={sri}"))
                .await
                .unwrap();
        Self {
            websocket: ws_stream,
            watched_games: vec![],
        }
    }

    pub async fn start_watching_current_games(&mut self) {
        let current_games = get_game_list(GameCategory::Best).await.unwrap();
        let cmd = Command {
            t: "startWatching".to_string(),
            d: current_games
                .iter()
                .map(|v| &v.id.0)
                .map(|v| format!("{v} "))
                .collect::<String>(),
        };
        self.websocket
            .send(Message::Text(serde_json::to_string(&cmd).unwrap()))
            .await
            .unwrap();
    }

    pub async fn recv_loop(&mut self) -> ! {
        loop {
            let msg = self.websocket.next().await;
            match msg {
                Some(Ok(v)) => println!("{v:?}"),
                Some(Err(why)) => println!("Error recving: {why}"),
                None => panic!("websocket closed"),
            }
        }
    }
}

#[derive(Serialize)]
struct Command {
    t: String,
    d: String,
}
