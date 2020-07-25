mod note;
mod note_pos;
mod optimize;
mod rate;
mod smparser;

use anyhow::{Context, Result};
use gnuplot::*;
use rate::{fatigues_at_notes, rate_notes, state::StepParams};
use smparser::{SMChart, SMResult};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,

    #[structopt(short = "v")]
    verbose: bool,

    #[structopt(parse(from_os_str), short = "g")]
    graph_path: Option<PathBuf>,

    #[structopt(short = "o")]
    optimize: bool,
}

fn graph(path: &PathBuf, vals: Vec<(String, Vec<f32>, Vec<f32>)>) {
    let mut fg = gnuplot::Figure::new();
    let a = fg
        .axes2d()
        .set_x_label("time", &[])
        .set_y_label("fatigue", &[]);
    for (name, times, fatigues) in vals {
        a.points(times, fatigues, &[PlotOption::Caption(&name)]);
    }

    fg.save_to_png(path, 1280, 720).unwrap();
}

fn rate(sms: &[SMResult], graph_path: Option<PathBuf>, step_params: &StepParams) -> Result<()> {
    let mut all_fatigues_over_time = Vec::new();
    let mut ratings = Vec::new();
    for sm in sms {
        for chart in &sm.0 {
            println!("Rating {}...", chart.full_name());
            let rating = rate_notes(&chart.notes, &step_params);
            ratings.push((chart, rating));
            if graph_path.is_some() {
                let fatigues = fatigues_at_notes(&chart.notes, &step_params);
                all_fatigues_over_time.push((
                    chart.full_name(),
                    chart.notes.iter().map(|n| n.time).collect(),
                    fatigues,
                ));
            }
        }
    }
    if let Some(graph_path) = graph_path {
        graph(&graph_path, all_fatigues_over_time);
    }
    ratings.sort_by(|(_, r1), (_, r2)| r1.partial_cmp(r2).unwrap());
    for (chart, rating) in ratings {
        println!("{}", chart.full_name());
        println!(" [{}] {}", chart.level, rating);
    }
    Ok(())
}

fn parse(paths: &[PathBuf], verbose: bool) -> Result<Vec<SMResult>> {
    let mut ret = Vec::new();

    for p in paths {
        let s = std::fs::read_to_string(p)?;
        let smresult =
            smparser::parse_sm(&s, verbose).with_context(|| format!("parsing {:?}", p))?;
        if !smresult.0.is_empty() {
            ret.push(smresult);
        }
    }
    Ok(ret)
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let sms = parse(&opts.inputs, opts.verbose)?;
    println!("parsed {} .sm files", sms.len());
    if opts.optimize {
        let step_params = optimize::optimize(
            &sms.clone()
                .into_iter()
                .flat_map(|sm| sm.0.into_iter())
                .collect::<Vec<SMChart>>(),
            32,
        )?;
        println!("found StepParams: {:?}", step_params);
        rate(&sms, opts.graph_path, &step_params)?;
    } else {
        rate(&sms, opts.graph_path, &StepParams::default())?;
    }
    Ok(())
}
