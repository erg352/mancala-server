#![allow(unused)]

#[derive(Clone, Copy)]
#[repr(align(8))]
pub struct Board([u8; 6]);

impl Board {
    #[inline]
    fn is_empty(&self) -> bool {
        // Code is optimized for performance, but might break if we
        // change the board struct.
        u64::from(*self) == 0
    }
}

impl From<Board> for u64 {
    #[inline]
    fn from(value: Board) -> Self {
        let mut buffer = [0u8; 8];
        buffer[..6].copy_from_slice(&value.0);
        u64::from_ne_bytes(buffer)
    }
}

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
        debug_assert!(player < 2 && cell < 6);

        todo!();
    }
}
