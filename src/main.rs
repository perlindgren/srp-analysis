use srp::common::Tasks;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(default_value=PathBuf::from("task_sets/task_set1.json").into_os_string())]
    path: PathBuf,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main() {
    let cli = Cli::parse();

    let tasks = Tasks::load(&cli.path).unwrap();
    println!("Task set\n{}", tasks);

    //     // println!("tasks {:?}", &tasks);
    println!("tot_util {}", tasks.total_utilization());
}
