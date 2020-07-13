use crate::note::Note;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct FootStatus {
    last_hit: Option<Note>,
    fatigue: f32,
}

impl FootStatus {
    fn new() -> Self {
        Self {
            last_hit: None,
            fatigue: 0.,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Foot {
    Left = 0,
    Right = 1,
}

impl Foot {
    pub fn other_foot(&self) -> Foot {
        match self {
            Foot::Left => Foot::Right,
            Foot::Right => Foot::Left,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct State {
    feet: [FootStatus; 2],
    fatigue: f32,
}

impl State {
    pub fn new() -> Self {
        Self {
            feet: [FootStatus::new(); 2],
            fatigue: 0.,
        }
    }
}

impl State {
    pub fn step(&self, foot: Foot, note: Note) -> State {
        let mut copy = *self;

        let mut foot = copy.feet[foot as usize];
        foot.fatigue += 0.5;
        foot.last_hit = Some(note);
        copy.fatigue += 0.5;
        copy
    }

    pub fn fatigue(&self) -> f32 {
        self.fatigue
    }
}

#[test]
fn test_state_step() {
    use crate::note::Pos;
    {
        let mut s = State::new();
        s = s.step(
            Foot::Left,
            Note {
                pos: Pos { x: 0., y: 0. },
                time: 0.0,
            },
        );
        s = s.step(
            Foot::Right,
            Note {
                pos: Pos { x: 0., y: 0. },
                time: 0.0,
            },
        );
        assert_eq!(s.fatigue, 1.);
    }
}
