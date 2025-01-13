use serde::Serialize;

pub mod play_match;
mod tests;

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

#[derive(Default, Debug)]
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

    pub fn is_move_valid(&self, player: u8, attempted_move: u8) -> bool {
        if self.boards[player as usize].is_empty() {
            if !(6..12).contains(&attempted_move) {
                false
            } else {
                self.boards[1 - player as usize].0[attempted_move as usize - 6] != 0
            }
        } else if !(0..6).contains(&attempted_move) {
            false
        } else {
            self.boards[player as usize].0[attempted_move as usize] != 0
        }
    }

    #[inline]
    pub fn play(&mut self, player: usize, cell: usize) -> usize {
        // Debugging stuff.
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
        debug_assert_ne!(stone_count, 0);

        self.boards[board_index].0[stone_index] = 0;

        stone_index += 1;
        for i in 0..stone_count {
            if stone_index < 6 {
                self.boards[board_index].0[stone_index] += 1;

                if board_index == player
                    && i == stone_count - 1
                    && self.boards[board_index].0[stone_index] == 1
                {
                    let stones_to_take = self.boards[1 - board_index].0[5 - stone_index];
                    self.boards[1 - board_index].0[5 - stone_index] = 0;

                    self.boards[board_index].0[stone_index] += stones_to_take;
                }
                stone_index += 1;

                continue;
            }

            board_index = 1 - board_index;

            if board_index == player {
                // We were in the opponement's board
                self.boards[board_index].0[0] += 1;
                stone_index = 1;
            } else {
                // We were in our board
                self.points[player] += 1;
                stone_index = 0;
                // We made a combo!
                if i == stone_count - 1 {
                    return player;
                }
            }
        }

        1 - player
    }
}
