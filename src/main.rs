use num_derive::FromPrimitive;
use png::OutputInfo;
use std::{collections::VecDeque, fs::File};

#[derive(Debug, PartialEq, FromPrimitive)]
enum PietColor {
    LightRed = 0xFFC0C0,
    LightYellow = 0xFFFFC0,
    LightGreen = 0xC0FFC0,
    LightCyan = 0xC0FFFF,
    LightBlue = 0xC0C0FF,
    LightMagenta = 0xFFC0FF,

    Red = 0xFF0000,
    Yellow = 0xFFFF00,
    Green = 0x00FF00,
    Cyan = 0x00FFFF,
    Blue = 0x0000FF,
    Magenta = 0xFF00FF,

    DarkRed = 0xC00000,
    DarkYellow = 0xC0C000,
    DarkGreen = 0x00C000,
    DarkCyan = 0x00C0C0,
    DarkBlue = 0x0000C0,
    DarkMagenta = 0xC000C0,

    Black = 0x000000,
    White = 0xFFFFFF,
}

impl From<&[u8]> for PietColor {
    fn from(bytes: &[u8]) -> Self {
        let sample = u32::from_be_bytes([0x0, bytes[0], bytes[1], bytes[2]]);
        let color: PietColor = num::FromPrimitive::from_u32(sample).unwrap();
        color
    }
}

enum CodelChoser {
    Left,
    Right,
}

enum DirectionPointer {
    Up,
    Down,
    Left,
    Right,
}

struct PietEnv<'a> {
    /// Direction Pointer
    dp: DirectionPointer,
    /// Codel Choser
    cc: CodelChoser,
    /// Codel Pointer
    cp: (usize, usize),
    /// Stores all data values
    stack: Vec<u8>,
    /// image
    image: &'a PietImg<'a>,
}

pub type Codel = (u32, u32);

struct PietImg<'a> {
    codel_size: u32,
    png_info: OutputInfo,
    bytes: &'a [u8],
}

#[derive(Debug)]
struct FloodFill {
    codels: Vec<Codel>,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
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

    pub fn get_codels_in_block(&self, loc: Codel) -> FloodFill {
        let mut queue = VecDeque::new();
        let mut seen = vec![];
        let mut min_y = self.png_info.height;
        let mut max_y = 0;
        let mut min_x = self.png_info.width;
        let mut max_x = 0;
        queue.push_back(loc);

        let block_color: PietColor = self[loc].into();
        eprintln!("Block color {:?}", block_color);

        while !queue.is_empty() {
            let n = queue.pop_front().unwrap();
            if seen.contains(&n) {
                continue;
            }
            let codel_color: PietColor = self[n].into();
            if codel_color != block_color {
                continue;
            }
            eprintln!("Codel color {:?} of {:?}", codel_color, n);

            if n.0 > max_x {
                max_x = n.0;
            }
            if n.1 > max_y {
                max_y = n.1;
            }
            if n.0 < min_x {
                min_x = n.0;
            }
            if n.1 < min_y {
                min_y = n.0;
            }
            seen.push(n);

            if n.0 > 0 {
                let left = (n.0 - 1, n.1);
                if !seen.contains(&left) {
                    queue.push_back(left)
                }
            }

            if n.1 > 0 {
                let up = (n.0, n.1 - 1);
                if !seen.contains(&up) {
                    queue.push_back(up);
                }
            }

            let right = (n.0 + 1, n.1);
            if n.0 + 1 < self.png_info.width && !seen.contains(&right) {
                queue.push_back(right);
            }

            let down = (n.0, n.1 + 1);
            if n.1 < self.png_info.height && !seen.contains(&down) {
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

    /// Get all the codels on *disjoint* edge in no order
    pub fn get_edge(&self, loc: Codel, dp: DirectionPointer) -> Vec<Codel> {
        let flood_fill = self.get_codels_in_block(loc);
        let mut edge = vec![];

        for node in flood_fill.codels {
            match dp {
                DirectionPointer::Right => {
                    if node.0 == flood_fill.max_x {
                        edge.push(node);
                    }
                }
                DirectionPointer::Left => {
                    if node.0 == flood_fill.min_x {
                        edge.push(node);
                    }
                }
                DirectionPointer::Down => {
                    if node.1 == flood_fill.max_y {
                        edge.push(node);
                    }
                }
                DirectionPointer::Up => {
                    if node.1 == flood_fill.min_y {
                        edge.push(node);
                    }
                }
            }
        }
        edge
    }
}

impl<'a> std::ops::Index<Codel> for PietImg<'a> {
    type Output = [u8];

    fn index(&self, loc: Codel) -> &Self::Output {
        let x = loc.0;
        let y = loc.1;
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
        let color: PietColor = num::FromPrimitive::from_u32(sample).unwrap();
    }
}

impl<'a> PietEnv<'a> {
    pub fn new(image: &'a PietImg) -> Self {
        PietEnv {
            dp: DirectionPointer::Right,
            cc: CodelChoser::Left,
            cp: (0, 0),
            stack: Vec::new(),
            image,
        }
    }

    fn step(&mut self) {
        // The interpreter finds the edge of the current colour block which is furthest in the direction of the DP. (This edge may be disjoint if the block is of a complex shape.)

        //            let self.image.get_edge(cp, dp);
        // The interpreter finds the codel of the current colour block on that edge which is furthest to the CC's direction of the DP's direction of travel. (Visualise this as standing on the program and walking in the direction of the DP; see table at right.)
        // The interpreter travels from that codel into the colour block containing the codel immediately in the direction of the DP.

        //           let next_block = self.imag
        //           let color = self.fetch_color(self.cp);
    }
}

fn main() -> Result<(), std::io::Error> {
    let decoder = png::Decoder::new(File::open("hello.png")?);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let bytes = &buf[..info.buffer_size()];

    // TODO verify more things about the PNG
    assert!(info.color_type == png::ColorType::Rgb);
    dbg!(&info);

    let image = PietImg::new(1, info, bytes);
    dbg!(image.get_edge((0, 0), DirectionPointer::Down));

    Ok(())
}

#[test]
fn assert_color_decode_in_one_codel_golden_image() {
    let decoder = png::Decoder::new(File::open("hello.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let image = PietImg::new(1, info, bytes);

    assert_eq!(PietColor::from(&image[(0, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[(1, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[(0, 1)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[(10, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[(11, 0)]), PietColor::DarkRed);
    assert_eq!(PietColor::from(&image[(18, 0)]), PietColor::Magenta);
    assert_eq!(PietColor::from(&image[(19, 0)]), PietColor::DarkMagenta);
    assert_eq!(PietColor::from(&image[(20, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[(21, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[(27, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[(29, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[(11, 1)]), PietColor::Magenta);
    assert_eq!(PietColor::from(&image[(19, 10)]), PietColor::Black);
    assert_eq!(PietColor::from(&image[(29, 24)]), PietColor::Green);
    assert_eq!(PietColor::from(&image[(29, 28)]), PietColor::LightYellow);
}

#[test]
fn flood_fill_test_in_one_codel_golden_image() {
    let decoder = png::Decoder::new(File::open("hello.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let image = PietImg::new(1, info, bytes);

    // the three vertical magenta blocks on the top row
    let flood_fill = image.get_codels_in_block((19, 0));
    assert_eq!(flood_fill.len(), 3);

    // the 4 pixel pyramid inside the main red start region
    let flood_fill = image.get_codels_in_block((4, 8));
    assert_eq!(flood_fill.len(), 4);

    // a black singleton cube
    let flood_fill = image.get_codels_in_block((4, 6));
    assert_eq!(flood_fill.len(), 1);
}
