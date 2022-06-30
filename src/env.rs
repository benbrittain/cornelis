use crate::image::PietImg;
use crate::ty::*;
use std::sync::Arc;

#[derive(Debug)]
pub enum PietOp {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Stack(Vec<u32>);

impl std::ops::DerefMut for Stack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Stack {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl druid::Data for Stack {
    fn same(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, x) in self.0.iter().enumerate() {
            write!(f, "{}:\t{:#02X}\n", i, x)?;
        }
        Ok(())
    }
}

#[derive(Clone, druid::Lens, PartialEq)]
pub struct PietEnv {
    /// Direction Pointer
    pub dp: DirectionPointer,
    /// Codel Choser
    pub cc: CodelChoser,
    /// Codel Pointer
    pub cp: Codel,
    /// Stores all data values
    pub stack: Stack,
    /// image
    pub image: PietImg,
    /// How many times we've hit a flow restriction (black blocks & edges)
    pub flow_restricted_count: usize,
    /// Output
    pub output: String,
}
impl druid::Data for PietEnv {
    fn same(&self, other: &Self) -> bool {
        self.dp == other.dp
            && self.cp == other.cp
            && self.output == other.output
            && self.cc == other.cc
    }
}

impl PietEnv {
    pub fn new(image: PietImg) -> Self {
        PietEnv {
            dp: DirectionPointer::Right,
            cc: CodelChoser::Left,
            cp: Codel { x: 0, y: 0 },
            stack: Stack(Vec::new()),
            flow_restricted_count: 0,
            image,
            output: String::new(),
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

    pub fn step(&mut self) {
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
                self.output.push(char::from_u32(val).unwrap().into());
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
            _ => {
                self.output.push_str(format!("UNKNOWN OP: {:?}", op).as_str());
                todo!()
            }
        }

        self.cp = next_node.unwrap();
    }
}
