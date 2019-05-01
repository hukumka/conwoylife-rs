use packed_simd::u8x64;

pub struct Life {
    size: BoardSize,
    current: Board,
    next: Board,
    triplets: Board,
}

impl Life {
    pub fn new(w: u32, h: u32) -> Self {
        let size = BoardSize {
            w,
            h,
            line_width: (w + 63) / 64,
        };
        Self {
            size,
            current: Board::new(size),
            next: Board::new(size),
            triplets: Board::new(size),
        }
    }

    pub fn value(&self) -> &Board{
        &self.current
    }

    pub fn update(&mut self) {
        self.calc_triplets();
        self.calc_next();
        std::mem::swap(&mut self.current, &mut self.next);
    }

    fn calc_next(&mut self) {
        // activate
        for i in 0..self.size.h{
            let sum = self.triplets.index(self.size, i, 0) + self.triplets.index(self.size, i, 1);
            self.activate_next(i, 0, sum);
            let sum = self.triplets.index(self.size, i, self.size.w-2) + self.triplets.index(self.size, i, self.size.w - 1);
            self.activate_next(i, self.size.w - 1, sum);
            for j in 1..self.size.w-1{
                let sum = self.triplets.index(self.size, i, j-1)
                    + self.triplets.index(self.size, i, j)
                    + self.triplets.index(self.size, i, j+1);
                self.activate_next(i, j, sum);
            }
        }
    }

    fn calc_triplets(&mut self) {
        if self.size.h < 3{
            unimplemented!("Cannot use height < 3");
        }
        for i in 0..self.size.line_width {
            let mut a = self.current.block_get(self.size, 0, i*64);
            let mut b = self.current.block_get(self.size, 1, i*64);
            let mut c = self.current.block_get(self.size, 2, i*64);
            self.triplets.block_set(self.size, 0, i, a + b);
            self.triplets.block_set(self.size, 1, i, b + c);
            for j in 2..self.size.h - 1 {
                a = b;
                b = c;
                c = self.current.block_get(self.size, j + 1, i*64);
                self.triplets.block_set(self.size, j, i*64, a + b + c);
            }
            self.triplets.block_set(self.size, self.size.h - 1, i*64,  b + c);
        }
    }

    #[inline]
    fn activate_next(&mut self, y: u32, x: u32, sum: u8) {
        let sum = sum - self.current.index(self.size, y, x);
        *self.next.index_mut(self.size, y, x) = if sum == 3 {
            1
        } else if sum != 2 {
            0
        } else {
            self.current.index(self.size, y, x)
        };
    }
}

#[derive(Copy, Clone)]
pub struct BoardSize {
    w: u32,
    h: u32,
    line_width: u32,
}

pub struct Board(Vec<u8>);

impl Board {
    fn new(size: BoardSize) -> Self {
        Board(vec![0; (size.line_width * size.h * 64) as usize])
    }

    #[inline]
    fn index(&self, size: BoardSize, y: u32, x: u32) -> u8 {
        self.0[(x + y * size.line_width) as usize]
    }

    #[inline]
    fn index_mut(&mut self, size: BoardSize, y: u32, x: u32) -> &mut u8 {
        &mut self.0[(x + y * size.line_width) as usize]
    }

    #[inline]
    fn block_get(&self, size: BoardSize, y: u32, x: u32) -> u8x64 {
        let start = (x + y * size.line_width) as usize;
        u8x64::from_slice_unaligned(&self.0[start..start+64])
    }

    #[inline]
    fn block_set(&mut self, size: BoardSize, y: u32, x: u32, value: u8x64){
        let start = (x + y * size.line_width) as usize;
        value.write_to_slice_unaligned(&mut self.0[start..start+64])
    }
}

