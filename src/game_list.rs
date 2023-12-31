use serde::Deserialize;

use crate::types::GameId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameCategory {
    Best,
}

impl GameCategory {
    pub fn get_name(&self) -> &'static str {
        match self {
            GameCategory::Best => "best",
        }
    }
}

pub async fn get_game_list(category: GameCategory) -> anyhow::Result<Vec<GameInfo>> {
    let text = reqwest::ClientBuilder::new()
        .build()?
        .get(format!(
            "https://lichess.org/api/tv/{}?nb=30",
            category.get_name()
        ))
        .header("Accept", "application/x-ndjson")
        .send()
        .await?
        .text()
        .await?;

    Ok(text
        .split('\n')
        .into_iter()
        .filter(|v| !v.is_empty())
        .map(serde_json::from_str)
        .map(Result::unwrap)
        .collect())
}

#[derive(Deserialize, Debug)]
pub struct GameInfo {
    pub id: GameId,
    rated: bool,
    variant: String,
    speed: String,
    perf: String,
    #[serde(rename = "createdAt")]
    created_at: u64,
    #[serde(rename = "lastMoveAt")]
    last_move_at: u64,
    status: String,
    players: Players,
    moves: String,
    clock: Clock,
}
#[derive(Deserialize, Debug)]
pub struct Players {
    white: Player,
    black: Player,
}

#[derive(Deserialize, Debug)]
pub struct Player {
    user: UserInfo,
    rating: u32,
}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    name: String,
    id: String,

    #[serde(default)]
    flair: Option<String>,

    #[serde(default)]
    title: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Clock {
    initial: u64,
    increment: i64,
    #[serde(rename = "totalTime")]
    total_time: u64,
}
