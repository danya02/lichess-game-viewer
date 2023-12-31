use serde::Deserialize;

#[derive(Deserialize, Debug, Hash)]
pub struct GameId(pub String);
