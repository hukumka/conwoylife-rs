use packed_simd::{shuffle, u8x64};
use rand::Rng;

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

    pub fn value(&self) -> u8{
        self.current.index(self.size, 0, 0)
    }

    pub fn new_random(w: u32, h: u32) -> Self {
        let mut l = Self::new(w, h);
        let mut rng = rand::thread_rng();
        for i in 0..h {
            for j in 0..w {
                *l.current.index_mut(l.size, i, j) = rng.gen_range(0, 2);
            }
        }
        l
    }

    pub fn update(&mut self) {
        self.calc_triplets();
        self.calc_next();
        std::mem::swap(&mut self.current, &mut self.next);
    }

    fn calc_next(&mut self) {
        for i in 0..self.size.h {
            for j in 0..self.size.line_width {
                let a = self.triplets.block_get_prev(self.size, i, j * 64);
                let b = self.triplets.block_get(self.size, i, j * 64);
                let c = self.triplets.block_get_next(self.size, i, j * 64);
                let total = a + b + c;
                let zero = u8x64::splat(0);
                let current = self.current.block_get(self.size, i, j * 64);
                // Trick to calculate next step without conditions
                let activation_mask = ((total - current) | current).eq(u8x64::splat(3));
                let activation = activation_mask.select(u8x64::splat(1), u8x64::splat(0));
                self.next.block_set(self.size, i, j * 64, activation);
            }
        }
    }

    fn calc_triplets(&mut self) {
        if self.size.h < 2 {
            unimplemented!("Cannot use height < 3");
        }
        for i in 0..self.size.line_width {
            let mut a = self.current.block_get(self.size, 0, i * 64);
            let mut b = self.current.block_get(self.size, 1, i * 64);
            let mut c;
            self.triplets.block_set(self.size, 0, i*64, a + b);
            for j in 1..self.size.h - 1 {
                c = self.current.block_get(self.size, j + 1, i * 64);
                self.triplets.block_set(self.size, j, i * 64, a + b + c);
                a = b;
                b = c;
            }
            self.triplets
                .block_set(self.size, self.size.h - 1, i * 64, a + b);
        }
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
        Board(vec![0; ((size.line_width*64 + 2) * size.h) as usize])
    }

    #[inline]
    fn index(&self, size: BoardSize, y: u32, x: u32) -> u8 {
        self.0[(x + 1 + y * (size.line_width * 64 + 2)) as usize]
    }

    #[inline]
    fn index_mut(&mut self, size: BoardSize, y: u32, x: u32) -> &mut u8 {
        &mut self.0[(x + 1 + y * (size.line_width * 64 + 2)) as usize]
    }

    #[inline]
    fn block_get(&self, size: BoardSize, y: u32, x: u32) -> u8x64 {
        let start = (x + 1 + y * (size.line_width * 64 + 2)) as usize;
        u8x64::from_slice_unaligned(&self.0[start..start + 64])
    }

    #[inline]
    fn block_get_prev(&self, size: BoardSize, y: u32, x: u32) -> u8x64 {
        let start = (x + y * (size.line_width * 64 + 2)) as usize;
        u8x64::from_slice_unaligned(&self.0[start..start + 64])
    }

    #[inline]
    fn block_get_next(&self, size: BoardSize, y: u32, x: u32) -> u8x64 {
        let start = (x + 2 + y * (size.line_width * 64 + 2)) as usize;
        u8x64::from_slice_unaligned(&self.0[start..start + 64])
    }

    #[inline]
    fn block_set(&mut self, size: BoardSize, y: u32, x: u32, value: u8x64) {
        let start = (x + 1 + y * (size.line_width * 64 + 2)) as usize;
        value.write_to_slice_unaligned(&mut self.0[start..start + 64])
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_update(){
        let mut life = Life::new_random(200, 200);
        *life.current.index_mut(life.size, 0, 0) = 1;
        *life.current.index_mut(life.size, 0, 1) = 1;
        *life.current.index_mut(life.size, 1, 1) = 1;
        *life.current.index_mut(life.size, 1, 0) = 1;
        life.update();
        for x in 0..life.size.w as i32{
            for y in 0..life.size.h as i32{
                let mut sum = 0;
                for i in -1..=1{
                    for j in -1..=1{
                        sum += index_next(&life, x+i, y+j);
                    }
                }
                let n = index_next(&life, x, y);
                sum -= n;
                if sum == 3{
                    assert_eq!(index_cur(&life, x, y), 1, "{} {}", x, y);
                } else if sum != 2{
                    assert_eq!(index_cur(&life, x, y), 0, "{} {}", x, y);
                } else{
                    assert_eq!(index_cur(&life, x, y), n, "{} {}", x, y);
                }
            }
        }
    }

    fn index_cur(life: &Life, x: i32, y: i32) -> u8{
        if x < 0 || x >= life.size.w as i32 || y < 0 || y >= life.size.h as i32{
            0
        }else{
            life.current.index(life.size, y as u32, x as u32)
        }
    }

    fn index_next(life: &Life, x: i32, y: i32) -> u8{
        if x < 0 || x >= life.size.w as i32 || y < 0 || y >= life.size.h as i32{
            0
        }else{
            life.next.index(life.size, y as u32, x as u32)
        }
    }
}
