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

fn train_until_plateau(
    charts: &[Chart],
    params: Params,
    max_iterations: i32,
    iteration: &mut i32,
) -> (Params, F1) {
    const LEARNING_RATE_INCREASE_MULTIPLIER: f64 = 1.1;
    const LEARNING_RATE_DECREASE_MULTIPLIER: f64 = 0.5;
    const INCREASE_LEARNING_RATE_AFTER_ITERATIONS: i32 = 10;
    const PLATEAU_ITERATIONS: i32 = 20;
    const PLATEAU_IMPROVEMENT: f64 = 0.002;
    let mut learning_rate = 0.001;
    let mut v = params.to_vec();
    let mut best_err = error(charts, Params::from_vec(&v));
    println!("initial err: {best_err}");
    let mut last_plateau_error = best_err;
    let mut check_plateau_iterations = PLATEAU_ITERATIONS;
    let mut iterations_since_last_learning_rate_change = 0;
    while *iteration < max_iterations {
        println!("iteration {iteration}, learning rate {learning_rate}");
        *iteration += 1;
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
            learning_rate *= LEARNING_RATE_DECREASE_MULTIPLIER;
            println!("new error higher than previous, retrying with smaller learning rate");
            iterations_since_last_learning_rate_change = 0;
            continue;
        }
        check_plateau_iterations -= 1;
        if check_plateau_iterations == 0 {
            if err > last_plateau_error - PLATEAU_IMPROVEMENT {
                println!("not enough improvement, bailing");
                break;
            }
            last_plateau_error = err;
            check_plateau_iterations = PLATEAU_ITERATIONS;
        }
        v = v_new;
        best_err = err;
        iterations_since_last_learning_rate_change += 1;
        if iterations_since_last_learning_rate_change > INCREASE_LEARNING_RATE_AFTER_ITERATIONS {
            iterations_since_last_learning_rate_change = 0;
            learning_rate *= LEARNING_RATE_INCREASE_MULTIPLIER;
            println!("increasing training rate");
        }
    }

    (Params::from_vec(&v), best_err)
}

fn mutate_params(mut params: Params) -> Params {
    use rand::distributions::{Distribution, Uniform};
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let range = Uniform::from(0.9..1.1);
    if rng.gen() {
        params.step_base *= range.sample(&mut rng);
    }
    if rng.gen() {
        params.step_dt_mult *= range.sample(&mut rng);
    }
    if rng.gen() {
        params.step_dt_add *= range.sample(&mut rng);
    }
    if rng.gen() {
        params.ratio_exp_base *= range.sample(&mut rng);
    }
    if rng.gen() {
        params.ratio_dt_mult *= range.sample(&mut rng);
    }
    params
}

pub fn train(charts: &[Chart], params: Params, max_iterations: i32) -> (Params, f64) {
    let mut iteration = 0;
    let (mut best_params, mut best_err) =
        train_until_plateau(charts, params, max_iterations, &mut iteration);
    while iteration < max_iterations {
        println!("mutating params and retrying");
        let (test_trained_params, test_trained_err) = train_until_plateau(
            charts,
            mutate_params(best_params),
            max_iterations,
            &mut iteration,
        );
        if test_trained_err < best_err {
            best_params = test_trained_params;
            best_err = test_trained_err;
        }
    }
    (best_params, best_err.x)
}
