use watcher::Watcher;

use crate::game_list::{get_game_list, GameCategory};

mod game_list;
mod types;
mod watcher;

#[tokio::main]
async fn main() {
    let mut watch = Watcher::new().await;
    watch.start_watching_current_games().await;
    watch.recv_loop().await;
}
