mod note;
mod note_pos;
mod rate;
mod smparser;

use anyhow::Result;
use rate::rate_notes;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Opts {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(short = "v")]
    verbose: bool,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let s = std::fs::read_to_string(opts.input)?;
    let smresult = smparser::parse_sm(&s, opts.verbose)?;
    println!("{}", smresult.title);
    let ratings = smresult.charts.iter().map(|n| rate_notes(&n.1));
    for r in ratings {
        println!("{}", r);
    }
    Ok(())
}
