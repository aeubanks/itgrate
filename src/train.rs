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
    let mut learning_rate = 0.001;
    let mut v = params_to_vec(params);
    let mut best_err = error(charts, vec_to_params(&v));
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
        let mut v_new = v.clone();
        for (x, g) in v_new.iter_mut().zip(grad.iter()) {
            *x -= g * learning_rate;
            *x = x.max(0.0);
        }
        println!("updated params: {:?}", &v_new);
        let err = error(charts, vec_to_params(&v_new));
        println!("err {}", err.value());
        if err > best_err {
            learning_rate *= 0.5;
            println!(
                "new error higher than previous, retrying with smaller learning rate {}",
                learning_rate
            );
            continue;
        }
        v = v_new;
        best_err = err;
    }

    vec_to_params(&v)
}
