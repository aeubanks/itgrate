use crate::chart::Chart;
use crate::rate::{rate, Params};
use autodiff::F1;

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
    let mut v = params.to_vec();
    let mut best_err = error(charts, Params::from_vec(&v));
    let mut iterations_since_last_learning_rate_change = 0;
    let increase_learning_rate_after_iterations = 10;
    for i in 0..iterations {
        println!("iteration {}, learning rate {}", i, learning_rate);
        let grad = autodiff::grad(
            |x| {
                error(
                    charts,
                    Params {
                        step_base: x[0],
                        step_dt_mult: x[1],
                        step_dt_add: x[2],
                        ratio_exp_base: x[3],
                        ratio_dt_mult: x[4],
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
        let err = error(charts, Params::from_vec(&v_new));
        println!("err {}", err.value());
        if err > best_err {
            learning_rate *= 0.5;
            println!("new error higher than previous, retrying with smaller learning rate");
            iterations_since_last_learning_rate_change = 0;
            continue;
        }
        v = v_new;
        best_err = err;
        iterations_since_last_learning_rate_change += 1;
        if iterations_since_last_learning_rate_change > increase_learning_rate_after_iterations {
            iterations_since_last_learning_rate_change = 0;
            learning_rate *= 1.1;
            println!("increasing training rate");
        }
    }

    Params::from_vec(&v)
}
