use crate::smparser::Chart;

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub step_1: f32,
    pub step_2: f32,
    pub linear: f32,
    pub exp_1: f32,
    pub exp_2: f32,
    pub recip_1: f32,
    pub recip_2: f32,
    pub sigmoid_1: f32,
    pub sigmoid_2: f32,
    pub tanh_1: f32,
}

struct State {
    cur_fatigue: f32,
    max_fatigue: f32,
    last_time: f32,
    params: Params,
}

impl State {
    fn with_params(params: Params) -> Self {
        Self {
            cur_fatigue: 0.,
            max_fatigue: 0.,
            last_time: 0.,
            params,
        }
    }
    fn step(&mut self, time: f32) {
        let dt = time - self.last_time;
        assert!(dt >= 0.);

        // ratio = 1 when dt = 0, ratio -> 0 as dt -> inf
        let ratio = (1.0 - self.params.linear * dt).max(0.0);
        // All of the below converge to ~.29 error
        // a^(-bx)
        // let ratio = self.params.exp_1.powf(-self.params.exp_2 * dt);
        // 1/(1+ax)^b
        // let ratio = 1.0 / (1.0 + self.params.recip_1 * dt).powf(self.params.recip_2);
        // 2-2/(1+a^(-bx))
        // let ratio = 2.0 - 2.0 / (1.0 + self.params.sigmoid_1.powf(-self.params.sigmoid_2 * dt));
        // 1-tanh(ax)
        // let ratio = 1.0 - (self.params.tanh_1 * dt).tanh();

        assert!(ratio >= 0.0);
        assert!(ratio <= 1.0);

        self.cur_fatigue *= ratio;
        self.cur_fatigue +=
            self.params.step_1 * (1.0 - self.params.step_2 + ratio * self.params.step_2);

        if self.cur_fatigue > self.max_fatigue {
            self.max_fatigue = self.cur_fatigue;
        }

        self.last_time = time;
    }
}

pub fn rate(chart: &Chart, params: Params) -> f32 {
    let mut fatigue = State::with_params(params);
    for note in &chart.notes {
        fatigue.step(note.time);
    }
    fatigue.max_fatigue
}
