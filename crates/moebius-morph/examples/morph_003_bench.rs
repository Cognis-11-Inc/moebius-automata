use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let report = match moebius_morph::emission::evaluate_morph003() {
        Ok(report) => report,
        Err(error) => {
            eprintln!("MORPH003_BENCH_FAIL {error}");
            return ExitCode::from(1);
        }
    };
    let root = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("runs/morph_003"));
    if let Err(error) = std::fs::create_dir_all(&root) {
        eprintln!("MORPH003_BENCH_FAIL {error}");
        return ExitCode::from(1);
    }
    let path = root.join("report.json");
    let json = match serde_json::to_string_pretty(&report) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("MORPH003_BENCH_FAIL {error}");
            return ExitCode::from(1);
        }
    };
    if let Err(error) = std::fs::write(&path, json) {
        eprintln!("MORPH003_BENCH_FAIL {error}");
        return ExitCode::from(1);
    }
    println!(
        "MORPH003_BENCH {} equal={} residual_faults={},{}",
        path.display(),
        report.experiment_f.equal,
        report.conservation_faults.duplicate_residual,
        report.conservation_faults.deletion_residual
    );
    ExitCode::SUCCESS
}
