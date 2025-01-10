#![allow(unused)]

use reqwest::Client;
use serde::Serialize;

#[derive(Clone, Copy, Default, Serialize)]
#[repr(align(8))]
pub struct Board([u8; 6]);

impl Board {
    fn is_empty(&self) -> bool {
        u64::from(*self) == 0
    }
}

impl From<Board> for u64 {
    fn from(value: Board) -> Self {
        // Code is optimized for performance, but might break if we
        // change the board struct.
        let mut buffer = [0u8; 8];
        buffer[..6].copy_from_slice(&value.0);
        u64::from_ne_bytes(buffer)
    }
}

#[derive(Default)]
pub struct Game {
    boards: [Board; 2],
    points: [u8; 2],
}

impl Game {
    #[inline]
    pub fn is_finished(&self) -> bool {
        // Code is optimized for performance, but might break if we
        // change the board struct.
        let mut buffer = [0u8; 16];
        buffer[..12].copy_from_slice(unsafe {
            std::slice::from_raw_parts(self.boards.as_ptr() as *const u8, 12)
        });
        u128::from_ne_bytes(buffer) == 0
    }

    #[inline]
    pub fn play(&mut self, player: usize, cell: usize) {
        // Debuggging stuff.
        debug_assert!(player < 2);
        if cfg!(debug_assertions) {
            if self.boards[player].is_empty() {
                debug_assert!((6..12).contains(&cell));
            } else {
                debug_assert!(cell < 6);
            }
        }

        todo!();
    }
}

impl Game {
    fn to_json(&self, player: usize) -> String {
        debug_assert!(player < 2);

        #[derive(Serialize)]
        struct SerializableGame {
            boards: [Board; 2],
            points: [u8; 2],
        }

        serde_json::to_string(&SerializableGame {
            boards: [self.boards[player], self.boards[1 - player]],
            points: [self.points[player], self.points[1 - player]],
        })
        .unwrap()
    }

    pub async fn send_game_to_player(&self, client: Client, player: usize, port: u16) {
        debug_assert!(player < 2);

        let serialized = self.to_json(player);
        let response = client
            .get(format!("localhost:{port}/next_move"))
            .body(serialized)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
    }
}
