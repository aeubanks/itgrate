#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Pos {
    pub x: f32,
    pub y: f32,
}

impl Pos {
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[test]
fn test_pos_distance() {
    let p1 = Pos { x: 1., y: 8. };
    let p2 = Pos { x: 4., y: 4. };
    assert!(p1.distance(&p2) - 5. < std::f32::EPSILON);
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Note {
    pub pos: Pos,
    pub time: f32,
}
