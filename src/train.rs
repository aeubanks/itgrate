use crate::chart::Chart;
use crate::rate::{rate, Params};
use autodiff::F1;

fn params_to_vec(params: Params) -> Vec<f64> {
    vec![
        params.step_1.value(),
        params.step_2.value(),
        params.dt_const.value(),
        params.ratio_1.value(),
        params.ratio_2.value(),
    ]
}

fn vec_to_params(v: &[f64]) -> Params {
    Params::new(v[0], v[1], v[2], v[3], v[4])
}

fn error(charts: &[Chart], params: Params) -> F1 {
    let mut error = F1::cst(0.);
    for chart in charts {
        let (rating, _) = rate(chart, params);
        let dr = rating - F1::cst(chart.rating as f32 + 0.5);
        error += dr * dr;
    }
    error / F1::cst(charts.len() as f32)
}

pub fn train(charts: &[Chart], params: Params, iterations: i32) -> Params {
    let mut learning_rate = 0.0001;
    let mut v = params_to_vec(params);
    for i in 0..iterations {
        println!("iteration {}", i);
        let grad = autodiff::grad(
            |x| {
                error(
                    charts,
                    Params {
                        step_1: x[0],
                        step_2: x[1],
                        dt_const: x[2],
                        ratio_1: x[3],
                        ratio_2: x[4],
                    },
                )
            },
            &v,
        );
        for (x, g) in v.iter_mut().zip(grad.iter()) {
            *x -= g * learning_rate;
        }
        v[2] = v[2].max(0.0);

        println!("updated params: {:?}", &v);
        let err = error(charts, vec_to_params(&v));
        println!("err {}", err.value());
        learning_rate *= 0.99;
    }

    vec_to_params(&v)
}
