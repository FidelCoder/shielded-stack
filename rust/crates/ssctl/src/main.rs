use std::process::ExitCode;

use bench::{empty_summary, BenchmarkPlan};
use lwd_client::{build_pending_report, Endpoint, Network};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return ExitCode::SUCCESS;
    };

    match command.as_str() {
        "health" => {
            let Some(url) = args.next() else {
                eprintln!("missing endpoint URL");
                return ExitCode::FAILURE;
            };

            let endpoint = Endpoint::new(Network::Mainnet, url);
            match build_pending_report(endpoint) {
                Ok(report) => {
                    println!("endpoint={}", report.endpoint.url);
                    println!("reachable={}", report.reachable);
                    println!("message={}", report.message);
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("invalid endpoint: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        "bench" => {
            let Some(url) = args.next() else {
                eprintln!("missing endpoint URL");
                return ExitCode::FAILURE;
            };

            let endpoint = Endpoint::new(Network::Mainnet, url);
            if let Err(err) = endpoint.validate() {
                eprintln!("invalid endpoint: {err}");
                return ExitCode::FAILURE;
            }

            let plan = BenchmarkPlan::new(endpoint);
            let summary = empty_summary(&plan);
            println!("attempted_requests={}", summary.attempted_requests);
            println!("successful_requests={}", summary.successful_requests);
            ExitCode::SUCCESS
        }
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("ssctl");
    println!();
    println!("Usage:");
    println!("  ssctl health <endpoint-url>");
    println!("  ssctl bench <endpoint-url>");
}
