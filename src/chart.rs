#[derive(PartialEq, Debug)]
pub struct Note {
    pub time: f32,
}

pub struct Chart {
    pub title: String,
    pub difficulty: String,
    pub notes: Vec<Note>,
    pub rating: i32,
}

impl Chart {
    pub fn description(&self) -> String {
        if self.difficulty.is_empty() {
            self.title.clone()
        } else {
            format!("{} ({})", self.title, self.difficulty)
        }
    }
}

impl Chart {
    pub fn from_unbroken(bpm: f32, measures: i32, rating: i32) -> Self {
        let num_notes = measures * 16;
        let mut notes = Vec::with_capacity(num_notes as usize);
        let dt = 15.0 / bpm;
        for i in 0..num_notes {
            notes.push(Note {
                time: dt * i as f32,
            });
        }
        Self {
            title: format!("{}@{}", measures, bpm),
            difficulty: "".to_owned(),
            notes,
            rating,
        }
    }
}

#[test]
fn test_from_unbroken() {
    let chart = Chart::from_unbroken(120.0, 2, 42);
    assert_eq!(chart.notes.len(), 32);
    assert_eq!(chart.notes[0], Note { time: 0.0 });
    assert_eq!(chart.notes[1], Note { time: 0.125 });
    assert_eq!(chart.notes[2], Note { time: 0.25 });
}
