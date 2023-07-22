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

    pub fn presets(only_longest: bool) -> Vec<Self> {
        let mut charts = vec![
            Chart::from_unbroken(170., 96, 15),
            Chart::from_unbroken(170., 128, 15),
            Chart::from_unbroken(170., 192, 16),
            Chart::from_unbroken(170., 256, 16),
            Chart::from_unbroken(170., 384, 17),
            Chart::from_unbroken(170., 512, 17),
            Chart::from_unbroken(180., 64, 15),
            Chart::from_unbroken(180., 96, 15),
            Chart::from_unbroken(180., 128, 16),
            Chart::from_unbroken(180., 192, 16),
            Chart::from_unbroken(180., 256, 17),
            Chart::from_unbroken(180., 384, 17),
            Chart::from_unbroken(180., 512, 18),
            Chart::from_unbroken(190., 48, 15),
            Chart::from_unbroken(190., 64, 15),
            Chart::from_unbroken(190., 96, 16),
            Chart::from_unbroken(190., 128, 17),
            Chart::from_unbroken(190., 192, 17),
            Chart::from_unbroken(190., 256, 18),
            Chart::from_unbroken(190., 384, 18),
            Chart::from_unbroken(190., 512, 19),
            Chart::from_unbroken(200., 32, 15),
            Chart::from_unbroken(200., 48, 15),
            Chart::from_unbroken(200., 64, 16),
            Chart::from_unbroken(200., 96, 17),
            Chart::from_unbroken(200., 128, 17),
            Chart::from_unbroken(200., 192, 18),
            Chart::from_unbroken(200., 256, 19),
            Chart::from_unbroken(200., 384, 19),
            Chart::from_unbroken(200., 512, 20),
            Chart::from_unbroken(210., 32, 15),
            Chart::from_unbroken(210., 48, 16),
            Chart::from_unbroken(210., 64, 17),
            Chart::from_unbroken(210., 96, 18),
            Chart::from_unbroken(210., 128, 18),
            Chart::from_unbroken(210., 192, 19),
            Chart::from_unbroken(210., 256, 20),
            Chart::from_unbroken(210., 384, 20),
            Chart::from_unbroken(210., 512, 21),
            Chart::from_unbroken(220., 32, 16),
            Chart::from_unbroken(220., 48, 17),
            Chart::from_unbroken(220., 64, 18),
            Chart::from_unbroken(220., 96, 19),
            Chart::from_unbroken(220., 128, 19),
            Chart::from_unbroken(220., 192, 20),
            Chart::from_unbroken(220., 256, 21),
            Chart::from_unbroken(220., 384, 22),
            Chart::from_unbroken(220., 512, 22),
            Chart::from_unbroken(230., 32, 17),
            Chart::from_unbroken(230., 48, 18),
            Chart::from_unbroken(230., 64, 19),
            Chart::from_unbroken(230., 96, 20),
            Chart::from_unbroken(230., 128, 20),
            Chart::from_unbroken(230., 192, 21),
            Chart::from_unbroken(230., 256, 22),
            Chart::from_unbroken(230., 384, 22),
            Chart::from_unbroken(230., 512, 23),
        ];

        if only_longest {
            charts.retain(|c| c.notes.len() == 512 * 16);
        }
        charts
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
