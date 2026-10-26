#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use moebius_cli::{RunOpts, run_batch, summary_json};

#[derive(Parser)]
#[command(
    name = "moebius",
    version,
    about = "Möbius Automata M0 headless batch runner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Commit a deterministic batch and write a manifest, metrics, trails, and snapshots.
    Run {
        /// Master seed (u64). It is not part of the configuration hash.
        #[arg(long)]
        seed: u64,
        /// Committed simulation ticks. Zero writes the initial state only.
        #[arg(long)]
        ticks: u64,
        /// Experiment configuration JSON.
        #[arg(long)]
        config: PathBuf,
        /// Output directory. Must be missing or empty. Existing output is not deleted.
        #[arg(long)]
        out: PathBuf,
        /// Ticks between snapshots. Tick 0 and the final tick are always included.
        #[arg(long, default_value_t = 50)]
        snapshot_interval: u64,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Run {
            seed,
            ticks,
            config,
            out,
            snapshot_interval,
        } => match run_batch(&RunOpts {
            seed,
            ticks,
            config,
            out,
            snapshot_interval,
        }) {
            Ok(report) => {
                println!("{}", summary_json(&report));
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("moebius: {err}");
                ExitCode::from(1)
            }
        },
    }
}
