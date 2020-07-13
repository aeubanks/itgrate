#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Note {
    pub pos: Pos,
    pub time: f32,
}
