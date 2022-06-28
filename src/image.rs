use crate::ty::*;
use png::OutputInfo;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct FloodFill {
    pub codels: Vec<Codel>,
    pub min_x: u32,
    pub min_y: u32,
    pub max_x: u32,
    pub max_y: u32,
}

pub struct PietImg<'a> {
    codel_size: u32,
    png_info: OutputInfo,
    bytes: &'a [u8],
}

impl<'a> PietImg<'a> {
    pub fn new(codel_size: u32, png_info: OutputInfo, bytes: &'a [u8]) -> Self {
        verify_colors(bytes);

        // being lazy for now
        assert!(codel_size == 1);

        PietImg {
            codel_size,
            png_info,
            bytes,
        }
    }

    pub fn contains(&self, loc: Codel) -> bool {
        loc.x < self.png_info.width && loc.y < self.png_info.height
    }

    pub fn get_codels_in_block(&self, loc: Codel) -> FloodFill {
        let mut queue = VecDeque::new();
        let mut seen: Vec<Codel> = vec![];
        let mut min_y = self.png_info.height;
        let mut max_y = 0;
        let mut min_x = self.png_info.width;
        let mut max_x = 0;
        queue.push_back(loc);

        let block_color: PietColor = self[loc].into();

        while !queue.is_empty() {
            let n = queue.pop_front().unwrap();
            if seen.contains(&n) {
                continue;
            }
            let codel_color: PietColor = self[n].into();
            if codel_color != block_color {
                continue;
            }

            if n.x > max_x {
                max_x = n.x;
            }
            if n.y > max_y {
                max_y = n.y;
            }
            if n.x < min_x {
                min_x = n.x;
            }
            if n.y < min_y {
                min_y = n.y;
            }
            seen.push(n);

            if n.x > 0 {
                let left = Codel::new(n.x - 1, n.y);
                if !seen.contains(&left) {
                    queue.push_back(left)
                }
            }

            if n.y > 0 {
                let up = Codel::new(n.x, n.y - 1);
                if !seen.contains(&up) {
                    queue.push_back(up);
                }
            }

            let right = Codel::new(n.x + 1, n.y);
            if n.x + 1 < self.png_info.width && !seen.contains(&right) {
                queue.push_back(right);
            }

            let down = Codel::new(n.x, n.y + 1);
            if n.y + 1 < self.png_info.height && !seen.contains(&down) {
                queue.push_back(down);
            }
        }

        FloodFill {
            codels: seen,
            max_x,
            min_x,
            max_y,
            min_y,
        }
    }
}

impl<'a> std::ops::Index<Codel> for PietImg<'a> {
    type Output = [u8];

    fn index(&self, loc: Codel) -> &Self::Output {
        let x = loc.x;
        let y = loc.y;
        assert!(x < self.png_info.width);
        assert!(y < self.png_info.height);
        let loc = ((y * (self.png_info.width * 3)) + x * 3) as usize;

        &self.bytes[loc..loc + 3]
    }
}

/// Ensure there are no colours we don't know how to handle in the png
fn verify_colors(bytes: &[u8]) {
    for chunk in bytes[..].chunks(3) {
        let sample = u32::from_be_bytes([0x0, chunk[0], chunk[1], chunk[2]]);
        let _: PietColor = num::FromPrimitive::from_u32(sample).unwrap();
    }
}
