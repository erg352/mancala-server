use reqwest::Client;
use serde::Serialize;

#[derive(Clone, Copy, Serialize, Debug, PartialEq)]
#[repr(align(8))]
pub struct Board([u8; 6]);

impl Default for Board {
    fn default() -> Self {
        Self([4; 6])
    }
}

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
    pub fn play(&mut self, player: usize, cell: usize) -> usize {
        // Debuggging stuff.
        debug_assert!(player < 2);

        if self.is_finished() {
            return player;
        }

        let (mut board_index, mut stone_index) = if self.boards[player].is_empty() {
            debug_assert!((6..12).contains(&cell));
            (1 - player, cell - 6)
        } else {
            debug_assert!(cell < 6);
            (player, cell)
        };

        let stone_count = self.boards[board_index].0[stone_index];
        self.boards[board_index].0[stone_index] = 0;

        let mut is_in_goal = false;

        stone_index += 1;
        for _ in 0..stone_count {
            is_in_goal = false;
            if stone_index < 6 {
                self.boards[board_index].0[stone_index] += 1;
                stone_index += 1;
                continue;
            } else {
                board_index = 1 - board_index;

                if board_index == player {
                    // We were in the opponement's board
                    self.boards[board_index].0[0] += 1;
                    stone_index = 1;
                } else {
                    // We were in our board
                    self.points[player] += 1;
                    stone_index = 0;
                    is_in_goal = true;
                }
            }
        }

        if is_in_goal {
            player
        } else {
            1 - player
        }
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
        let _response = client
            .get(format!("localhost:{port}/next_move"))
            .body(serialized)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play() {
        let mut game = Game::default();

        let player = game.play(0, 2);

        assert_eq!(player, 0);
        assert_eq!(game.boards[1], Board([4, 4, 4, 4, 4, 4]));
        assert_eq!(game.boards[0], Board([4, 4, 0, 5, 5, 5]));

        assert_eq!(game.points[1], 0);
        assert_eq!(game.points[0], 1);
    }
}
