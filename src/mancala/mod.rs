use std::ops::{Deref, DerefMut};

use serde::Serialize;

pub mod play_match;
mod tests;

#[derive(Clone, Copy, Serialize, Debug, PartialEq)]
#[repr(align(8))]
pub struct Board([u8; 6]);

impl Default for Board {
    #[inline]
    fn default() -> Self {
        Self([4; 6])
    }
}

impl Deref for Board {
    type Target = [u8; 6];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Board {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Board {
    #[inline]
    fn is_empty(&self) -> bool {
        u64::from(*self) == 0
    }
}

impl From<Board> for u64 {
    #[inline]
    fn from(value: Board) -> Self {
        // Code is optimized for performance, but might break if we
        // change the board struct.
        let mut buffer = [0u8; 8];
        buffer[..6].copy_from_slice(&*value);
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

    #[inline]
    pub fn is_move_valid(&self, player: u8, attempted_move: u8) -> bool {
        if self.boards[player as usize].is_empty() {
            if !(6..12).contains(&attempted_move) {
                false
            } else {
                self.boards[1 - player as usize][attempted_move as usize - 6] != 0
            }
        } else if !(0..6).contains(&attempted_move) {
            false
        } else {
            self.boards[player as usize][attempted_move as usize] != 0
        }
    }

    pub fn play(&mut self, player: usize, cell: usize) -> usize {
        // There are only two players, so we must be sure the player index is 0 or 1.
        debug_assert!(player < 2);

        // If both boards are completely empty, then the game is finished. It doesn't matter
        // whether we return player or 1 - player, as we will no longer be querying the bots.
        if self.is_finished() {
            return player;
        }

        // Get the board index and stone_index (the index in the board) based off of the cell
        // and the current player
        let (mut board_index, mut stone_index) = if self.boards[player].is_empty() {
            // There are no more pieces in our board, we have to look at our opponent's board.
            debug_assert!((6..12).contains(&cell));
            (1 - player, cell - 6)
        } else {
            // There are still pieces in our board, we must look at our own board (can't be our
            // opponent's board).
            debug_assert!(cell < 6);
            (player, cell)
        };

        // Get the stones from the player's target cell. There must be at least one stone at that
        // position in order for this to be valid. This should be checked before calling this
        // function.
        let stone_count = self.boards[board_index][stone_index];
        debug_assert_ne!(stone_count, 0);

        // Pick up all the stones from the player's desired cell.
        self.boards[board_index][stone_index] = 0;

        // We start dropping down the stones at the next index.
        stone_index += 1;
        for i in 0..stone_count {
            // So long as the stone_index < 6, we can continue dropping down stones in the current
            // board.
            if stone_index < 6 {
                self.boards[board_index][stone_index] += 1;

                // If we are currently in the player's board, on the last stone to drop and the
                // cell we are dropping to used to be empty, than we take all stones from our
                // opponent's facing cell.
                if board_index == player
                    && i == stone_count - 1
                    && self.boards[board_index][stone_index] == 1
                {
                    // Get the amount of stones to take from our opponent cell. Because boards are
                    // flipped, we use 5 - stone_index and not just stone_index.
                    let stones_to_take = self.boards[1 - board_index][5 - stone_index];
                    // We clear the stones from our opponent's board (they are now in our board).
                    self.boards[1 - board_index][5 - stone_index] = 0;

                    // We add the stones to our board.
                    self.boards[board_index][stone_index] += stones_to_take;
                }
                // We must move to the next stone_index for our next pass.
                stone_index += 1;

                // The following code is for when stone_index >= 6, and we can thus safely exit
                // (as we are in the branch stone_index < 6)
                continue;
            }

            // Because we are now in the other board, we switch the board index.
            board_index = 1 - board_index;

            // If we used to be in the opponent's board, we want to directly go to our own board.
            if board_index == player {
                // We directly add to the first cell in our board.
                self.boards[board_index][0] += 1;
                // The stone index is now 1 for the next pass.
                stone_index = 1;
            } else {
                // If we used to be in our board, we first want to add 1 to our score before
                // starting to go through the opponent's board.
                self.points[player] += 1;
                // The stone index is now 0 for the next pass (because we never added to the
                // opponent's board).
                stone_index = 0;
                // If we scored with the last stone in our hand, we made a combo, and as such we
                // can play another time.
                if i == stone_count - 1 {
                    return player;
                }
            }
        }

        // The round was ended normaly (without a combo). As such, it is now the opponent's turn.
        1 - player
    }
}
