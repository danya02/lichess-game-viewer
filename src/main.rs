use std::{collections::HashMap, str::FromStr};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::OriginDimensions,
    pixelcolor::{Rgb888, RgbColor},
};
use pixels::{Pixels, SurfaceTexture};
use shakmaty::{fen::Fen, Chess};
use tokio::sync::mpsc;
use watcher::Watcher;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use crate::{
    drawing::draw_chesses,
    game_list::{get_game_list, GameCategory},
};

mod drawing;
mod game_list;
mod types;
mod watcher;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct ArrayDrawTarget<'a> {
    target: &'a mut [u8],
    width: usize,
}

impl<'a> DrawTarget for ArrayDrawTarget<'a> {
    type Color = Rgb888;

    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        for embedded_graphics::Pixel(pos, color) in pixels {
            if pos.x < 0
                || pos.x >= self.width as i32
                || pos.y < 0
                || pos.y >= (self.target.len() / self.width) as i32
            {
                continue;
            }
            let offset = (pos.y as usize * self.width + pos.x as usize) * 4;
            if offset + 4 > self.target.len() {
                return Ok(());
            }
            self.target[offset..offset + 4].copy_from_slice(&[
                color.r(),
                color.g(),
                color.b(),
                255,
            ]);
        }

        Ok(())
    }
}

impl<'a> OriginDimensions for ArrayDrawTarget<'a> {
    fn size(&self) -> embedded_graphics::prelude::Size {
        embedded_graphics::prelude::Size::new(
            self.width as u32,
            (self.target.len() / self.width) as u32,
        )
    }
}

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
    };

    let (send, mut recv) = mpsc::channel(100);

    let mut watch = Watcher::new(send).await;
    watch.start_watching_current_games().await;
    // watch.pump_replacements_until_count(50).await;

    println!("{:?}", watch.watched_games);
    println!("{:?}", watch.watched_games.len());
    tokio::spawn(async move { watch.recv_loop().await });

    tokio::task::block_in_place(move || {
        let mut games = vec![];
        let mut game_states = HashMap::new();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { window_id, event } => {
                    if let WindowEvent::Resized(new_size) = event {
                        pixels
                            .resize_surface(new_size.width, new_size.height)
                            .unwrap();
                        pixels
                            .resize_buffer(new_size.width, new_size.height)
                            .unwrap();
                    }
                }
                Event::RedrawRequested(_) => {
                    let mut needs_redraw = false;
                    while let Ok(v) = recv.try_recv() {
                        needs_redraw = true;
                        match v {
                            types::GameEvent::GameListUpdate(new_games) => {
                                games = new_games;
                                for id in &games {
                                    if !game_states.contains_key(id) {
                                        game_states
                                            .insert(id.clone(), shakmaty::fen::Fen::default());
                                    }
                                }
                            }
                            types::GameEvent::GameEvent(ev) => match ev {
                                types::LichessWebsocketEvent::Finish { id, win } => {}
                                types::LichessWebsocketEvent::Fen {
                                    id,
                                    lm,
                                    fen,
                                    wc,
                                    bc,
                                } => {
                                    if !game_states.contains_key(&id) {
                                        game_states
                                            .insert(id.clone(), shakmaty::fen::Fen::default());
                                    }

                                    *game_states.get_mut(&id).unwrap() =
                                        Fen::from_str(&fen).unwrap()
                                }
                            },
                        }
                        println!("Should redraw? {needs_redraw}");
                        if needs_redraw {
                            let mut target = ArrayDrawTarget {
                                target: pixels.frame_mut(),
                                width: window.inner_size().width as usize,
                            };

                            draw_chesses(&mut target, &games, &game_states);
                            pixels.render().unwrap();
                        }
                    }
                }
                _ => {}
            };
            window.request_redraw();
        });
    });
}
