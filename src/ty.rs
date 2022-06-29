use num_derive::FromPrimitive;
use druid::{Lens, Data};

#[derive(Data, Debug, Copy, Clone, PartialEq, FromPrimitive)]
pub enum PietColor {
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
    pub fn get_color_scale(&self) -> (u32, u32) {
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

#[derive(Data, Debug, PartialEq, Clone, Copy)]
pub enum CodelChoser {
    Left,
    Right,
}

#[derive(Data, Debug, PartialEq, Clone, Copy)]
pub enum DirectionPointer {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Data, PartialEq, Debug, Clone, Copy)]
pub struct Codel {
    pub x: u32,
    pub y: u32,
}

impl Codel {
    pub fn new(x: u32, y: u32) -> Self {
        Codel { x, y }
    }

    pub fn block_in_dir(self, other: DirectionPointer) -> Option<Self> {
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
