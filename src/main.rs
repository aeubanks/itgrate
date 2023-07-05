mod smparser;

use clap::Parser;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[structopt(help = "Paths of/directories containing .sm files")]
    inputs: Vec<PathBuf>,
}

fn sm_files_impl(path: &PathBuf, set: &mut HashSet<PathBuf>) {
    let metadata = std::fs::metadata(path).expect("couldn't get metadata for path");
    if metadata.is_file() {
        if let Some(Some(ext)) = path.extension().map(|e| e.to_str()) {
            if ext.to_lowercase() == "sm" {
                set.insert(path.clone());
            }
        }
    } else if metadata.is_dir() {
        for de in std::fs::read_dir(path).expect("couldn't read entries in directory") {
            sm_files_impl(&de.unwrap().path(), set);
        }
    }
}

fn sm_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut set = HashSet::new();
    for path in paths {
        sm_files_impl(path, &mut set);
    }
    set.into_iter().collect()
}

fn main() {
    let args = Args::parse();

    let sm_files = sm_files(&args.inputs);

    if sm_files.is_empty() {
        println!("Didn't find any sm files");
        std::process::exit(1);
    }

    for sm_file in sm_files {
        println!("Reading {:?}", sm_file);
        let buf = std::fs::read(sm_file).unwrap();
        let str = std::str::from_utf8(&buf).unwrap();
        let charts = smparser::parse(str);
        for chart in charts {
            println!(
                "{} - {} ({}) has {} notes",
                chart.title,
                chart.difficulty,
                chart.rating,
                chart.notes.len()
            )
        }
    }
}
