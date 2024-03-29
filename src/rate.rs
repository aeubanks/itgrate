use crate::chart::Chart;
use autodiff::{Float, Zero, F1};

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub step_dt_mult: F1,
    pub step_dt_add: F1,
    pub ratio_dt_mult: F1,
}

impl Params {
    pub fn new(step_dt_mult: f64, step_dt_add: f64, ratio_dt_mult: f64) -> Self {
        Self {
            step_dt_mult: F1::cst(step_dt_mult),
            step_dt_add: F1::cst(step_dt_add),
            ratio_dt_mult: F1::cst(ratio_dt_mult),
        }
    }

    pub fn to_vec(self) -> Vec<f64> {
        vec![
            self.step_dt_mult.value(),
            self.step_dt_add.value(),
            self.ratio_dt_mult.value(),
        ]
    }

    pub fn from_vec(v: &[f64]) -> Self {
        Self::new(v[0], v[1], v[2])
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new(1.6725047878328008, 22.69176212395888, 0.03094850290834286)
    }
}

struct State {
    cur_fatigue: F1,
    max_fatigue: F1,
    last_time: f64,
    params: Params,
}

impl State {
    fn with_params(params: Params) -> Self {
        Self {
            cur_fatigue: F1::cst(0.0),
            max_fatigue: F1::cst(0.0),
            last_time: 0.,
            params,
        }
    }
    fn step(&mut self, time: f64) {
        let dt = F1::cst(time - self.last_time);
        assert!(dt.value() >= 0.);

        let ratio = if dt.is_zero() {
            F1::cst(1.0)
        } else {
            (-self.params.ratio_dt_mult * dt).exp()
        };

        if ratio.value() < 0.0 || ratio.value() > 1.0 {
            panic!("unexpected ratio: {}, dt {}", ratio, dt);
        }

        self.cur_fatigue *= ratio;
        self.cur_fatigue +=
            F1::cst(1.0) / (dt * self.params.step_dt_mult + self.params.step_dt_add);

        if self.cur_fatigue > self.max_fatigue {
            self.max_fatigue = self.cur_fatigue;
        }

        self.last_time = time;
    }
}

pub fn rate(chart: &Chart, params: Params) -> (F1, Vec<(f64, f64)>) {
    let mut fatigue = State::with_params(params);
    let mut fatigues = Vec::with_capacity(chart.notes.len());
    fatigues.push((0.0, 0.0));
    for note in &chart.notes {
        fatigue.step(note.time);
        fatigues.push((note.time, fatigue.cur_fatigue.value()))
    }
    (fatigue.max_fatigue, fatigues)
}

#[test]
fn rate_sanity_tests() {
    let params = Params::default();
    assert!(
        rate(&Chart::stream_with_arrowless_break(200., 16, 0), params)
            < rate(&Chart::stream_with_8ths_break(200., 16, 0), params)
    );
    assert!(
        rate(&Chart::stream_unbroken(200., 16, 0), params)
            < rate(&Chart::stream_with_arrowless_break(200., 16, 0), params)
    );
    assert!(
        rate(&Chart::stream_unbroken(200., 16, 0), params)
            < rate(&Chart::stream_unbroken(200., 17, 0), params)
    );
    assert!(
        rate(&Chart::stream_unbroken(200., 16, 0), params)
            < rate(&Chart::stream_unbroken(201., 16, 0), params)
    );
}
