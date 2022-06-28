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

impl<'a> PietImg<'a> {
    pub fn new(codel_size: u32, png_info: OutputInfo, bytes: &'a [u8]) -> Self {
        verify_colors(bytes);

        // being lazy for now
        assert!(codel_size == 1);

        //// ensure the codel size works with the dimensions of the png
        //assert!(dims.0 % codel_size == 0);
        //assert!(dims.1 % codel_size == 0);

        PietImg {
            codel_size,
            png_info,
            bytes,
        }
    }

    pub fn get_codels_in_block(&self, loc: Codel) -> Vec<Codel> {
        let mut queue = VecDeque::new();
        let mut seen = vec![];
        queue.push_back(loc);

        let block_color: PietColor = self[loc].into();
        eprintln!("Block color {:?}", block_color);

        while !queue.is_empty() {
            let n = queue.pop_front().unwrap();
            eprintln!("checking: {:?}", n);
            if seen.contains(&n) {
                continue;
            }
            let codel_color: PietColor = self[n].into();
            eprintln!("Codel color {:?} of {:?}", codel_color, n);
            if codel_color != block_color {
                continue;
            }

            dbg!(&n, codel_color);
            seen.push(n);

            if n.0 > 0 {
                let left = (n.0 - 1, n.1);
                if !seen.contains(&left){
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
            dbg!(&queue);


            //let codel_color: PietColor = self[n].into();
            //if codel_color == block_color {
            //    dbg!(&n);
            //    dbg!(&codel_color);
            //    if n.0 + 1 < self.png_info.width && !seen.contains(&(n.0 + 1, n.1)) {
            //        queue.push_back((n.0 + 1, n.1));
            //    }
            //    if n.0 > 0 && !seen.contains(&(n.0 - 1, n.1)) {
            //        queue.push_back((n.0 - 1, n.1));
            //    }
            //    if n.1 + 1 < self.png_info.height && !seen.contains(&(n.0, n.1 + 1)) {
            //        queue.push_back((n.0, n.1 + 1));
            //    }
            //    if n.1 > 0 && !seen.contains(&(n.0, n.1 - 1)) {
            //        queue.push_back((n.0, n.1 - 1));
            //    }
            //}
        }

        seen
    }

    pub fn get_edge(&self, loc: Codel, dp: DirectionPointer) -> Vec<(Codel, Codel)> {
        vec![]
    }
}

impl<'a> std::ops::Index<Codel> for PietImg<'a> {
    type Output = [u8];

    fn index(&self, loc: Codel) -> &Self::Output {
        let x = loc.0;
        let y = loc.1;
        let loc = ((y * (self.png_info.width * 3)) + x * 3) as usize;
        eprintln!("indexing {} {} @ {}", x, y, loc);

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
    ////    let piet = PietEnv::new(&image);
    //dbg!(image.get_codels_in_block((8, 27)).len());

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
