use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{connect, Message},
};

use crate::{
    game_list::{get_game_list, get_game_replacement, GameCategory},
    types::{GameEvent, GameId, LichessWebsocketEvent},
};

enum WatcherCommand {
    /// Replace one of the games that was watched.
    ReplaceGame(GameId),
}

pub struct Watcher {
    websocket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    cmd_recv: mpsc::Receiver<WatcherCommand>,
    cmd_send: mpsc::Sender<WatcherCommand>,
    pub watched_games: Vec<GameId>,
    ev_send: mpsc::Sender<GameEvent>,
}

impl Watcher {
    pub async fn new(game_event_send: mpsc::Sender<GameEvent>) -> Self {
        let mut rng = thread_rng();
        let mut sri = String::new();
        for _ in 0..12 {
            sri.push(rng.sample(rand::distributions::Alphanumeric) as char);
        }
        let (ws_stream, _) =
            connect_async(format!("wss://socket2.lichess.org/socket/v5?sri={sri}"))
                .await
                .unwrap();
        let (send, recv) = mpsc::channel(100);
        Self {
            websocket: ws_stream,
            cmd_recv: recv,
            cmd_send: send,
            watched_games: vec![],
            ev_send: game_event_send,
        }
    }

    pub async fn start_watching_one(&mut self, id: GameId) {
        let cmd = Command {
            t: "startWatching".to_string(),
            d: id.0.clone(),
        };
        self.websocket
            .send(Message::Text(serde_json::to_string(&cmd).unwrap()))
            .await
            .unwrap();
        self.watched_games.push(id);
        self.ev_send
            .send(GameEvent::GameListUpdate(self.watched_games.clone()))
            .await
            .unwrap();
    }

    pub async fn start_watching_one_instead(&mut self, id: GameId, instead_of_id: GameId) {
        for game in self.watched_games.iter_mut() {
            if game.0 == instead_of_id.0 {
                let cmd = Command {
                    t: "startWatching".to_string(),
                    d: id.0.clone(),
                };
                self.websocket
                    .send(Message::Text(serde_json::to_string(&cmd).unwrap()))
                    .await
                    .unwrap();

                *game = id;
                return;
            }
        }
        self.ev_send
            .send(GameEvent::GameListUpdate(self.watched_games.clone()))
            .await
            .unwrap();
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
        self.watched_games
            .extend(current_games.into_iter().map(|v| v.id));
        self.ev_send
            .send(GameEvent::GameListUpdate(self.watched_games.clone()))
            .await
            .unwrap();
    }

    pub async fn pump_replacements_until_count(&mut self, target_count: usize) {
        while self.watched_games.len() < target_count {
            let next_game = get_game_replacement(
                GameCategory::Best,
                &self.watched_games[0],
                &self.watched_games,
            )
            .await
            .unwrap();
            self.start_watching_one(next_game).await;
        }
    }

    pub async fn recv_loop(&mut self) -> ! {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            tokio::select! {
                msg = self.websocket.next() => self.handle_message(msg).await,
                cmd = self.cmd_recv.recv() => self.handle_command(cmd).await,
                _ = interval.tick() => {
                    self.websocket.send(Message::Text("null".to_string())).await.unwrap();
                }
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
    ) {
        match msg {
            Some(Ok(m)) => match m {
                Message::Text(t) => {
                    if t != "0" {
                        let msg: LichessWebsocketEvent = serde_json::from_str(&t).unwrap();
                        println!("{msg:?}");
                        match &msg {
                            LichessWebsocketEvent::Finish { id, win } => {
                                async fn replace_game_later(
                                    which: GameId,
                                    sender: mpsc::Sender<WatcherCommand>,
                                ) {
                                    tokio::time::sleep(Duration::from_secs(3)).await;
                                    sender
                                        .send(WatcherCommand::ReplaceGame(which))
                                        .await
                                        .unwrap();
                                }

                                tokio::spawn(replace_game_later(id.clone(), self.cmd_send.clone()));
                            }
                            _ => {}
                        }
                        self.ev_send.send(GameEvent::GameEvent(msg)).await.unwrap();
                    }
                }
                Message::Binary(what) => panic!("Unexpected binary msg: {what:?}"),
                Message::Close(what) => panic!("Unexpected close msg: {what:?}"),
                _ => {}
            },
            Some(Err(why)) => panic!("Error in Lichess websocket stream: {why}"),
            None => panic!("Lichess websocket stream ran out"),
        }
    }

    async fn handle_command(&mut self, cmd: Option<WatcherCommand>) {
        match cmd {
            None => panic!("All command senders closed!"),
            Some(cmd) => match cmd {
                WatcherCommand::ReplaceGame(which) => {
                    let replacement =
                        get_game_replacement(GameCategory::Best, &which, &self.watched_games)
                            .await
                            .unwrap();
                    self.start_watching_one_instead(replacement, which).await;
                }
            },
        }
    }
}

#[derive(Serialize)]
struct Command {
    t: String,
    d: String,
}
