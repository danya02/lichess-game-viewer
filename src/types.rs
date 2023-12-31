use serde::Deserialize;

#[derive(Deserialize, Debug, Hash, Clone, PartialEq, Eq)]
pub struct GameId(pub String);

pub enum GameEvent {
    GameListUpdate(Vec<GameId>),
    GameEvent(LichessWebsocketEvent),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "t", content = "d", rename_all = "lowercase")]
pub enum LichessWebsocketEvent {
    Finish {
        id: GameId,
        win: Option<String>,
    },
    Fen {
        id: GameId,
        lm: String,
        fen: String,
        wc: u32,
        bc: u32,
    },
}
