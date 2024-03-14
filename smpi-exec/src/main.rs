//! Local node parallel process executor.
use std::process;
use clap::Parser;
use std::thread;

const DEFAULT_PROC_COUNT: u64 = 2;

#[derive(Parser)]
struct Args {
    /// Number of processes to spawn on this node
    #[arg(short)]
    proc_count: Option<u64>,

    /// Binary to run
    binary: String,

    /// Arguments to binary
    args: Vec<String>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let proc_count = args.proc_count.unwrap_or(DEFAULT_PROC_COUNT);
    let mut children = vec![];
    for proc_id in 0..proc_count {
        log::info!("starting process {}", proc_id);
        let child = process::Command::new(&args.binary)
            .args(&args.args)
            .spawn()
            .expect("failed to spawn child program");
        children.push(child);
    }

    // TODO: Set up communication mechanism with child processes

    let bg_thread = thread::spawn(move || {
        // TODO: Wait for requests from children
    });

    // Wait for all children
    for (proc_id, child) in children.iter_mut().enumerate() {
        let status = child.wait().expect("failed to await process");
        log::info!("child process {} completed with {}", proc_id, status);
    }

    bg_thread.join();
}
