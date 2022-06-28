use num_derive::FromPrimitive;
use png::OutputInfo;
use std::{collections::VecDeque, fs::File};

#[derive(Debug, Copy, Clone, PartialEq, FromPrimitive)]
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

impl PietColor {
    fn get_color_scale(&self) -> (u32, u32) {
        match self {
            PietColor::LightRed => (0, 0),
            PietColor::Red => (0, 1),
            PietColor::DarkRed => (0, 2),
            PietColor::LightYellow => (1, 0),
            PietColor::Yellow => (1, 1),
            PietColor::DarkYellow => (1, 2),
            PietColor::LightGreen => (2, 0),
            PietColor::Green => (2, 1),
            PietColor::DarkGreen => (2, 2),
            PietColor::LightCyan => (3, 0),
            PietColor::Cyan => (3, 1),
            PietColor::DarkCyan => (3, 2),
            PietColor::LightBlue => (4, 0),
            PietColor::Blue => (4, 1),
            PietColor::DarkBlue => (4, 2),
            PietColor::LightMagenta => (5, 0),
            PietColor::Magenta => (5, 1),
            PietColor::DarkMagenta => (5, 2),
            _ => panic!("not on the hue/light cycle!"),
        }
    }
}

impl From<&[u8]> for PietColor {
    fn from(bytes: &[u8]) -> Self {
        let sample = u32::from_be_bytes([0x0, bytes[0], bytes[1], bytes[2]]);
        let color: PietColor = num::FromPrimitive::from_u32(sample).unwrap();
        color
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum CodelChoser {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    cp: Codel,
    /// Stores all data values
    stack: Vec<u32>,
    /// image
    image: &'a PietImg<'a>,
    /// How many times we've hit a flow restriction (black blocks & edges)
    flow_restricted_count: usize,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Codel {
    x: u32,
    y: u32,
}

impl Codel {
    pub fn new(x: u32, y: u32) -> Self {
        Codel { x, y }
    }

    fn block_in_dir(self, other: DirectionPointer) -> Option<Self> {
        Some(match other {
            DirectionPointer::Right => Codel {
                x: self.x.checked_add(1)?,
                y: self.y,
            },
            DirectionPointer::Left => Codel {
                x: self.x.checked_sub(1)?,
                y: self.y,
            },
            DirectionPointer::Up => Codel {
                x: self.x,
                y: self.y.checked_sub(1)?,
            },
            DirectionPointer::Down => Codel {
                x: self.x,
                y: self.y.checked_add(1)?,
            },
        })
    }
}

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
                let left = Codel { x: n.x - 1, y: n.y };
                if !seen.contains(&left) {
                    queue.push_back(left)
                }
            }

            if n.y > 0 {
                let up = Codel { x: n.x, y: n.y - 1 };
                if !seen.contains(&up) {
                    queue.push_back(up);
                }
            }

            let right = Codel { x: n.x + 1, y: n.y };
            if n.x + 1 < self.png_info.width && !seen.contains(&right) {
                queue.push_back(right);
            }

            let down = Codel { x: n.x, y: n.y + 1 };
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
        let color: PietColor = num::FromPrimitive::from_u32(sample).unwrap();
    }
}

#[derive(Debug)]
enum PietOp {
    None,
    Push,
    Pop,

    Add,
    Subtract,
    Multiply,

    Divide,
    Mod,
    Not,

    Greater,
    Pointer,
    Switch,

    Duplicate,
    Roll,
    InNumber,

    InChar,
    OutNumber,
    OutChar,
}

fn get_op(node: PietColor, next_node: PietColor) -> PietOp {
    let color = &node.get_color_scale();
    let next_color = &next_node.get_color_scale();
    let darkness = (next_color.1 + 3 - color.1) % 3;
    let hue = (next_color.0 + 6 - color.0) % 6;

    if darkness == 0 && hue == 0 {
        PietOp::None
    } else if darkness == 1 && hue == 0 {
        PietOp::Push
    } else if darkness == 2 && hue == 5 {
        PietOp::OutChar
    } else if darkness == 0 && hue == 4 {
        PietOp::Duplicate
    } else if darkness == 1 && hue == 2 {
        PietOp::Multiply
    } else {
        panic!(
            "Unknown transition of {:?} {:?} ({:?})",
            node,
            next_node,
            (darkness, hue)
        );
    }
}

impl<'a> PietEnv<'a> {
    pub fn new(image: &'a PietImg) -> Self {
        PietEnv {
            dp: DirectionPointer::Right,
            cc: CodelChoser::Left,
            cp: Codel { x: 0, y: 0 },
            stack: Vec::new(),
            flow_restricted_count: 0,
            image,
        }
    }

    /// Get all the codels on *disjoint* edge in no order
    pub fn get_block_transition(&self, loc: Codel, dp: DirectionPointer) -> (Codel, u32) {
        let flood_fill = self.image.get_codels_in_block(loc);
        dbg!(&flood_fill.codels);
        dbg!(dp);
        dbg!(flood_fill.min_x, flood_fill.min_y);
        dbg!(flood_fill.max_x, flood_fill.max_y);

        // 1. The interpreter finds the edge of the current colour block which is furthest in the direction of the DP. (This edge may be disjoint if the block is of a complex shape.)
        let mut edge = vec![];
        for node in &flood_fill.codels {
            match dp {
                DirectionPointer::Right => {
                    if node.x == flood_fill.max_x {
                        edge.push(node);
                    }
                }
                DirectionPointer::Left => {
                    if node.x == flood_fill.min_x {
                        edge.push(node);
                    }
                }
                DirectionPointer::Down => {
                    if node.y == flood_fill.max_y {
                        edge.push(node);
                    }
                }
                DirectionPointer::Up => {
                    if node.y == flood_fill.min_y {
                        edge.push(node);
                    }
                }
            }
        }

        dbg!(&edge);
        // 2. The interpreter finds the codel of the current colour block on that edge which
        // is furthest to the CC's direction of the DP's direction of travel.
        // (Visualise this as standing on the program and walking in the direction of the DP; see table at right.)
        let exit_node = if self.dp == DirectionPointer::Right && self.cc == CodelChoser::Left {
            edge.sort_unstable_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            // Uppermost
            edge[0]
        } else if self.dp == DirectionPointer::Right && self.cc == CodelChoser::Right {
            edge.sort_unstable_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            // Lowermost
            edge.last().unwrap()
        } else if self.dp == DirectionPointer::Down && self.cc == CodelChoser::Right {
            edge.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            // leftmost
            edge[0]
        } else if self.dp == DirectionPointer::Down && self.cc == CodelChoser::Left {
            edge.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            // rightmost
            edge.last().unwrap()
        } else if self.dp == DirectionPointer::Left && self.cc == CodelChoser::Left {
            edge.sort_unstable_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            // lowermost
            edge.last().unwrap()
        } else if self.dp == DirectionPointer::Left && self.cc == CodelChoser::Right {
            edge.sort_unstable_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            // uppermost
            edge[0]
        } else if self.dp == DirectionPointer::Up && self.cc == CodelChoser::Left {
            edge.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            // leftmost
            edge[0]
        } else if self.dp == DirectionPointer::Up && self.cc == CodelChoser::Right {
            edge.sort_unstable_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            // rightmost
            edge.last().unwrap()
        } else {
            todo!()
        };

        (*exit_node, flood_fill.codels.len() as u32)
    }

    fn step(&mut self) {
        eprintln!("====== STEP ======");
        let (exit_node, node_size) = self.get_block_transition(self.cp, self.dp);

        let loc_color: PietColor = self.image[self.cp].into();
        let node_color: PietColor = self.image[exit_node].into();
        assert_eq!(loc_color, node_color);
        // The interpreter travels from that codel into the colour block containing the codel immediately in the direction of the DP.
        let next_node = exit_node.block_in_dir(self.dp);

        let next_node_color = if let Some(next_node) = next_node {
            if self.image.contains(next_node) {
                self.image[next_node].into()
            } else {
                PietColor::Black
            }
        } else {
            PietColor::Black
        };

        if next_node_color == PietColor::Black {
            if self.flow_restricted_count >= 8 {
                println!("PROGRAM EXECUTION COMPLETE");
                return;
            }
            if self.flow_restricted_count % 2 == 0 {
                match self.cc {
                    CodelChoser::Left => self.cc = CodelChoser::Right,
                    CodelChoser::Right => self.cc = CodelChoser::Left,
                }
            } else if self.flow_restricted_count % 2 == 1 {
                match self.dp {
                    DirectionPointer::Right => self.dp = DirectionPointer::Down,
                    DirectionPointer::Down => self.dp = DirectionPointer::Left,
                    DirectionPointer::Left => self.dp = DirectionPointer::Up,
                    DirectionPointer::Up => self.dp = DirectionPointer::Right,
                }
            }

            println!("BLACK RESTRICT #{}", self.flow_restricted_count);
            self.flow_restricted_count += 1;
            eprintln!(
                "{:?} | {:?}/{:?} => {:?}/{:?} # CC {:?} # DP {:?}",
                self.cp, exit_node, node_color, next_node, next_node_color, self.cc, self.dp,
            );
            return;
        }

        // decode the transition
        let op = get_op(node_color, next_node_color);
        self.flow_restricted_count = 0;
        eprintln!(
            "{:?} | {:?}/{:?} => {:?}/{:?} [{:?}]",
            self.cp, exit_node, node_color, next_node, next_node_color, op
        );

        match op {
            PietOp::Push => self.stack.push(node_size),
            PietOp::OutChar => {
                let val = self.stack.pop().unwrap();
                println!("OUT: {}", char::from_u32(val).unwrap());
            }
            PietOp::Duplicate => {
                let val = self.stack.pop().unwrap();
                self.stack.push(val);
                self.stack.push(val);
            }
            PietOp::Multiply => {
                let a = self.stack.pop().unwrap();
                let b = self.stack.pop().unwrap();
                self.stack.push(a * b);
            }
            _ => todo!(),
        }

        self.cp = next_node.unwrap();
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

    let image = PietImg::new(1, info, bytes);
    let mut env = PietEnv::new(&image);

    env.step();

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

    assert_eq!(PietColor::from(&image[Codel::new(0, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[Codel::new(1, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[Codel::new(0, 1)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[Codel::new(10, 0)]), PietColor::Red);
    assert_eq!(
        PietColor::from(&image[Codel::new(11, 0)]),
        PietColor::DarkRed
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(18, 0)]),
        PietColor::Magenta
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(19, 0)]),
        PietColor::DarkMagenta
    );
    assert_eq!(PietColor::from(&image[Codel::new(20, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[Codel::new(21, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[Codel::new(27, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[Codel::new(29, 0)]), PietColor::Blue);
    assert_eq!(
        PietColor::from(&image[Codel::new(11, 1)]),
        PietColor::Magenta
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(19, 10)]),
        PietColor::Black
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(29, 24)]),
        PietColor::Green
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(29, 28)]),
        PietColor::LightYellow
    );
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
    let flood_fill = image.get_codels_in_block(Codel::new(19, 0));
    assert_eq!(flood_fill.codels.len(), 3);

    // the 4 pixel pyramid inside the main red start region
    let flood_fill = image.get_codels_in_block(Codel::new(4, 8));
    assert_eq!(flood_fill.codels.len(), 4);

    // a black singleton cube
    let flood_fill = image.get_codels_in_block(Codel::new(4, 6));
    assert_eq!(flood_fill.codels.len(), 1);

    let flood_fill = image.get_codels_in_block(Codel::new(25, 25));
}
