use std::fs;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bench::{run_benchmark, BenchmarkPlan};
use lwd_client::{
    probe_or_report, Endpoint, EndpointRegistry, EndpointStatus, Network, ProbeOptions,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Serialize)]
struct StatusDocument {
    updated_at: String,
    summary: StatusSummary,
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize)]
struct StatusSummary {
    total_services: usize,
    reachable_services: usize,
    degraded_services: usize,
    unreachable_services: usize,
}

#[derive(Debug, Serialize)]
struct ServiceStatus {
    id: String,
    network: Network,
    url: String,
    status: EndpointStatus,
    region: String,
    operator: String,
    reachable: bool,
    latest_block_height: Option<u64>,
    estimated_block_height: Option<u64>,
    height_lag: Option<u64>,
    latency_ms: u128,
    checked_at_unix: u64,
    message: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        return ExitCode::SUCCESS;
    }

    match args[0].as_str() {
        "health" => health_command(&args[1..]).await,
        "bench" => bench_command(&args[1..]).await,
        "registry" => registry_command(&args[1..]).await,
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

async fn health_command(args: &[String]) -> ExitCode {
    let mut network = Network::Mainnet;
    let mut timeout = Duration::from_secs(10);
    let mut output = OutputFormat::Human;
    let mut url = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--network" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --network");
                    return ExitCode::FAILURE;
                };
                network = match parse_network(value) {
                    Ok(network) => network,
                    Err(message) => {
                        eprintln!("{message}");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--timeout-seconds" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --timeout-seconds");
                    return ExitCode::FAILURE;
                };
                timeout = match parse_duration(value) {
                    Ok(timeout) => timeout,
                    Err(message) => {
                        eprintln!("{message}");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--json" => output = OutputFormat::Json,
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                return ExitCode::FAILURE;
            }
            value => url = Some(value.to_string()),
        }
        index += 1;
    }

    let Some(url) = url else {
        eprintln!("missing endpoint URL");
        return ExitCode::FAILURE;
    };

    let report = probe_or_report(Endpoint::new(network, url), ProbeOptions { timeout }).await;
    print_health_report(&report, output);

    if report.reachable {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

async fn bench_command(args: &[String]) -> ExitCode {
    let mut network = Network::Mainnet;
    let mut timeout = Duration::from_secs(10);
    let mut requests = 10;
    let mut output = OutputFormat::Human;
    let mut url = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--network" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --network");
                    return ExitCode::FAILURE;
                };
                network = match parse_network(value) {
                    Ok(network) => network,
                    Err(message) => {
                        eprintln!("{message}");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--requests" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --requests");
                    return ExitCode::FAILURE;
                };
                requests = match value.parse::<u32>() {
                    Ok(0) => 1,
                    Ok(value) => value,
                    Err(_) => {
                        eprintln!("--requests must be a positive integer");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--timeout-seconds" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --timeout-seconds");
                    return ExitCode::FAILURE;
                };
                timeout = match parse_duration(value) {
                    Ok(timeout) => timeout,
                    Err(message) => {
                        eprintln!("{message}");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--json" => output = OutputFormat::Json,
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                return ExitCode::FAILURE;
            }
            value => url = Some(value.to_string()),
        }
        index += 1;
    }

    let Some(url) = url else {
        eprintln!("missing endpoint URL");
        return ExitCode::FAILURE;
    };

    let endpoint = Endpoint::new(network, url);
    if let Err(err) = endpoint.validate() {
        eprintln!("invalid endpoint: {err}");
        return ExitCode::FAILURE;
    }

    let summary = run_benchmark(
        BenchmarkPlan::new(endpoint)
            .with_requests(requests)
            .with_timeout(timeout),
    )
    .await;

    match output {
        OutputFormat::Human => {
            println!("endpoint={}", summary.endpoint_url);
            println!("attempted_requests={}", summary.attempted_requests);
            println!("successful_requests={}", summary.successful_requests);
            println!("failed_requests={}", summary.failed_requests);
            println!("fastest_ms={}", optional_u128(summary.fastest_ms));
            println!("slowest_ms={}", optional_u128(summary.slowest_ms));
            println!("average_ms={}", optional_u128(summary.average_ms));
            println!(
                "latest_block_height={}",
                optional_u64(summary.latest_block_height)
            );
        }
        OutputFormat::Json => print_json(&summary),
    }

    if summary.successful_requests > 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

async fn registry_command(args: &[String]) -> ExitCode {
    let Some(subcommand) = args.first() else {
        eprintln!("missing registry subcommand");
        print_help();
        return ExitCode::FAILURE;
    };

    match subcommand.as_str() {
        "validate" => registry_validate_command(&args[1..]),
        "probe" => registry_probe_command(&args[1..]).await,
        "status" => registry_status_command(&args[1..]).await,
        other => {
            eprintln!("unknown registry subcommand: {other}");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn registry_validate_command(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("missing registry path");
        return ExitCode::FAILURE;
    };

    match load_registry(path) {
        Ok(registry) => {
            println!("registry={path}");
            println!("network={:?}", registry.network);
            println!("endpoints={}", registry.endpoints.len());
            println!("active_endpoints={}", registry.active_endpoints().count());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

async fn registry_probe_command(args: &[String]) -> ExitCode {
    let mut timeout = Duration::from_secs(10);
    let mut output = OutputFormat::Human;
    let mut path = None;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--timeout-seconds" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --timeout-seconds");
                    return ExitCode::FAILURE;
                };
                timeout = match parse_duration(value) {
                    Ok(timeout) => timeout,
                    Err(message) => {
                        eprintln!("{message}");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--json" => output = OutputFormat::Json,
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                return ExitCode::FAILURE;
            }
            value => path = Some(value.to_string()),
        }
        index += 1;
    }

    let Some(path) = path else {
        eprintln!("missing registry path");
        return ExitCode::FAILURE;
    };

    let registry = match load_registry(&path) {
        Ok(registry) => registry,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let mut reports = Vec::new();
    for endpoint in registry.active_endpoints() {
        reports.push(probe_or_report(endpoint, ProbeOptions { timeout }).await);
    }

    match output {
        OutputFormat::Human => {
            println!("registry={path}");
            println!("reports={}", reports.len());
            for report in &reports {
                println!();
                print_health_report(report, OutputFormat::Human);
            }
        }
        OutputFormat::Json => print_json(&reports),
    }

    if reports.iter().all(|report| report.reachable) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

async fn registry_status_command(args: &[String]) -> ExitCode {
    let mut timeout = Duration::from_secs(10);
    let mut output_path = None;
    let mut registry_paths = Vec::new();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--timeout-seconds" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --timeout-seconds");
                    return ExitCode::FAILURE;
                };
                timeout = match parse_duration(value) {
                    Ok(timeout) => timeout,
                    Err(message) => {
                        eprintln!("{message}");
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--output" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("missing value for --output");
                    return ExitCode::FAILURE;
                };
                output_path = Some(value.to_string());
            }
            value if value.starts_with('-') => {
                eprintln!("unknown option: {value}");
                return ExitCode::FAILURE;
            }
            value => registry_paths.push(value.to_string()),
        }
        index += 1;
    }

    if registry_paths.is_empty() {
        eprintln!("missing registry path");
        return ExitCode::FAILURE;
    }

    let mut registries = Vec::new();
    for path in &registry_paths {
        match load_registry(path) {
            Ok(registry) => registries.push(registry),
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::FAILURE;
            }
        }
    }

    let document = build_status_document(&registries, timeout).await;
    let output = match serde_json::to_string_pretty(&document) {
        Ok(output) => format!("{output}\n"),
        Err(err) => {
            eprintln!("failed to serialize status JSON: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Some(path) = output_path {
        if let Err(err) = fs::write(&path, output) {
            eprintln!("failed to write {path}: {err}");
            return ExitCode::FAILURE;
        }
        println!("wrote={path}");
    } else {
        print!("{output}");
    }

    ExitCode::SUCCESS
}

async fn build_status_document(
    registries: &[EndpointRegistry],
    timeout: Duration,
) -> StatusDocument {
    let mut services = Vec::new();

    for registry in registries {
        for record in &registry.endpoints {
            let report = if record.status == EndpointStatus::Active {
                Some(
                    probe_or_report(
                        Endpoint::new(registry.network, record.url.clone()),
                        ProbeOptions { timeout },
                    )
                    .await,
                )
            } else {
                None
            };

            let height_lag = report.as_ref().and_then(|report| {
                match (report.latest_block_height, report.estimated_block_height) {
                    (Some(latest), Some(estimated)) => Some(estimated.saturating_sub(latest)),
                    _ => None,
                }
            });

            services.push(ServiceStatus {
                id: record.id.clone(),
                network: registry.network,
                url: record.url.clone(),
                status: record.status,
                region: record.region.clone(),
                operator: record.operator.clone(),
                reachable: report
                    .as_ref()
                    .map(|report| report.reachable)
                    .unwrap_or(false),
                latest_block_height: report
                    .as_ref()
                    .and_then(|report| report.latest_block_height),
                estimated_block_height: report
                    .as_ref()
                    .and_then(|report| report.estimated_block_height),
                height_lag,
                latency_ms: report.as_ref().map(|report| report.latency_ms).unwrap_or(0),
                checked_at_unix: report
                    .as_ref()
                    .map(|report| report.checked_at_unix)
                    .unwrap_or_else(unix_time),
                message: report
                    .map(|report| report.message)
                    .unwrap_or_else(|| "not active".to_string()),
            });
        }
    }

    let total_services = services.len();
    let reachable_services = services.iter().filter(|service| service.reachable).count();
    let degraded_services = services
        .iter()
        .filter(|service| service.reachable && service.height_lag.unwrap_or(0) > 0)
        .count();
    let unreachable_services = services
        .iter()
        .filter(|service| service.status == EndpointStatus::Active && !service.reachable)
        .count();

    StatusDocument {
        updated_at: utc_now_string(),
        summary: StatusSummary {
            total_services,
            reachable_services,
            degraded_services,
            unreachable_services,
        },
        services,
    }
}

fn load_registry(path: &str) -> Result<EndpointRegistry, String> {
    let contents =
        fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))?;
    EndpointRegistry::from_yaml(&contents).map_err(|err| format!("invalid registry {path}: {err}"))
}

fn parse_network(value: &str) -> Result<Network, String> {
    match value {
        "mainnet" => Ok(Network::Mainnet),
        "testnet" => Ok(Network::Testnet),
        other => Err(format!("unknown network: {other}")),
    }
}

fn parse_duration(value: &str) -> Result<Duration, String> {
    let seconds = value
        .parse::<u64>()
        .map_err(|_| "--timeout-seconds must be a positive integer".to_string())?;
    Ok(Duration::from_secs(seconds.max(1)))
}

fn print_health_report(report: &lwd_client::HealthReport, output: OutputFormat) {
    match output {
        OutputFormat::Json => print_json(report),
        OutputFormat::Human => {
            println!("endpoint={}", report.endpoint.url);
            println!("network={:?}", report.endpoint.network);
            println!("reachable={}", report.reachable);
            println!(
                "latest_block_height={}",
                optional_u64(report.latest_block_height)
            );
            println!("latency_ms={}", report.latency_ms);
            println!("vendor={}", optional_str(report.vendor.as_deref()));
            println!("version={}", optional_str(report.version.as_deref()));
            println!("chain_name={}", optional_str(report.chain_name.as_deref()));
            println!("message={}", report.message);
        }
    }
}

fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(output) => println!("{output}"),
        Err(err) => eprintln!("failed to serialize JSON: {err}"),
    }
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn optional_u128(value: Option<u128>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn optional_str(value: Option<&str>) -> &str {
    value.unwrap_or("none")
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn utc_now_string() -> String {
    let output = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "1970-01-01T00:00:00Z".to_string(),
    }
}

fn print_help() {
    println!("ssctl");
    println!();
    println!("Usage:");
    println!(
        "  ssctl health <endpoint-url> [--network mainnet|testnet] [--timeout-seconds n] [--json]"
    );
    println!("  ssctl bench <endpoint-url> [--network mainnet|testnet] [--requests n] [--timeout-seconds n] [--json]");
    println!("  ssctl registry validate <path>");
    println!("  ssctl registry probe <path> [--timeout-seconds n] [--json]");
    println!("  ssctl registry status <path>... [--timeout-seconds n] [--output path]");
}
