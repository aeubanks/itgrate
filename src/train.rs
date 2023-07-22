use crate::chart::Chart;
use crate::rate::{rate, Params};

pub fn train(charts: &[Chart], mut params: Params, hill_climb_iterations: i32) -> Params {
    let mut err = error(charts, params);

    let mut rng = rand::thread_rng();

    for i in 0..hill_climb_iterations {
        let mut new_params = params;
        new_params.rand(&mut rng);
        let new_err = error(charts, new_params);
        if new_err < err {
            params = new_params;
            err = new_err;
            println!("iteration {}", i);
            println!("better params: {:?}, err {}", params, err);
        }
    }

    println!("--------");
    println!("params: {:?}", params);
    println!("err {}", err);

    params
}

fn error(charts: &[Chart], params: Params) -> f32 {
    let mut error = 0.;
    for chart in charts {
        let (rating, _) = rate(chart, params);
        let dr = rating - (chart.rating as f32 + 0.5);
        error += dr * dr;
    }
    error / charts.len() as f32
}
