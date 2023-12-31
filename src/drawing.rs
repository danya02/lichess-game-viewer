use std::collections::HashMap;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Primitive, PrimitiveStyleBuilder},
};
use shakmaty::fen::Fen;

use crate::types::GameId;

const CHESS_WIDTH: u32 = 96;
const BUFF: u32 = 32;

pub fn draw_chesses<D: DrawTarget<Color = Rgb888> + OriginDimensions>(
    to: &mut D,
    games: &[GameId],
    states: &HashMap<GameId, Fen>,
) where
    <D as DrawTarget>::Error: std::fmt::Debug,
{
    to.fill_solid(&to.bounding_box(), Rgb888::new(0, 0, 0))
        .unwrap();

    let mut start_x = 0;
    let mut start_y = 0;
    for id in games {
        draw_single_chess(to, states[id].clone(), start_x, start_y);
        start_x += CHESS_WIDTH as i32 + BUFF as i32;
        if start_x + CHESS_WIDTH as i32 > to.size().width as i32 {
            start_x = 0;
            start_y += CHESS_WIDTH as i32 + BUFF as i32;
        }
    }
}

pub fn draw_single_chess<D: DrawTarget<Color = Rgb888> + OriginDimensions>(
    to: &mut D,
    state: Fen,
    start_x: i32,
    start_y: i32,
) where
    <D as DrawTarget>::Error: std::fmt::Debug,
{
    // Draw the 8x8 squares
    let square_width = CHESS_WIDTH / 8;
    let isquare_width: i32 = square_width as i32;
    for i in 0..8 {
        for j in 0..8 {
            embedded_graphics::primitives::Rectangle::new(
                Point::new(start_x + isquare_width * i, start_y + isquare_width * j),
                Size::new(square_width, square_width),
            )
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(if (i + j) % 2 > 0 {
                        Rgb888::new(0x86, 0xA6, 0x66)
                    } else {
                        Rgb888::new(0xFF, 0xFF, 0xDD)
                    })
                    .build(),
            )
            .draw(to)
            .unwrap();
        }
    }

    // Draw the pieces on the chessboard.
    // (currently just draws circles on the points)

    for piece in state.0.board.occupied().into_iter() {
        let (file, rank) = piece.coords();
        let x = file.distance(shakmaty::File::A) as i32;
        let y = rank.distance(shakmaty::Rank::Eighth) as i32;
        embedded_graphics::primitives::Circle::new(
            Point::new(start_x + isquare_width * x, start_y + isquare_width * y),
            square_width,
        )
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(if state.0.board.white().contains(piece) {
                    Rgb888::new(255, 255, 255)
                } else {
                    Rgb888::new(0, 0, 0)
                })
                .build(),
        )
        .draw(to)
        .unwrap();
    }
}
