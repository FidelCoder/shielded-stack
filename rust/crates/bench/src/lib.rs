use std::time::Duration;

use lwd_client::{probe_or_report, Endpoint, HealthReport, ProbeOptions};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkPlan {
    pub endpoint: Endpoint,
    pub requests: u32,
    pub timeout: Duration,
}

impl BenchmarkPlan {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            requests: 10,
            timeout: Duration::from_secs(10),
        }
    }

    pub fn with_requests(mut self, requests: u32) -> Self {
        self.requests = requests.max(1);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BenchmarkSummary {
    pub endpoint_url: String,
    pub attempted_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub fastest_ms: Option<u128>,
    pub slowest_ms: Option<u128>,
    pub average_ms: Option<u128>,
    pub latest_block_height: Option<u64>,
    pub reports: Vec<HealthReport>,
}

pub async fn run_benchmark(plan: BenchmarkPlan) -> BenchmarkSummary {
    let mut reports = Vec::with_capacity(plan.requests as usize);

    for _ in 0..plan.requests {
        let report = probe_or_report(
            plan.endpoint.clone(),
            ProbeOptions {
                timeout: plan.timeout,
            },
        )
        .await;
        reports.push(report);
    }

    summarize(plan.endpoint.url, plan.requests, reports)
}

pub fn summarize(
    endpoint_url: String,
    attempted_requests: u32,
    reports: Vec<HealthReport>,
) -> BenchmarkSummary {
    let successful: Vec<&HealthReport> = reports.iter().filter(|report| report.reachable).collect();
    let successful_requests = successful.len() as u32;
    let failed_requests = attempted_requests.saturating_sub(successful_requests);
    let fastest_ms = successful.iter().map(|report| report.latency_ms).min();
    let slowest_ms = successful.iter().map(|report| report.latency_ms).max();
    let average_ms = if successful.is_empty() {
        None
    } else {
        Some(
            successful
                .iter()
                .map(|report| report.latency_ms)
                .sum::<u128>()
                / successful.len() as u128,
        )
    };
    let latest_block_height = successful
        .iter()
        .filter_map(|report| report.latest_block_height)
        .max();

    BenchmarkSummary {
        endpoint_url,
        attempted_requests,
        successful_requests,
        failed_requests,
        fastest_ms,
        slowest_ms,
        average_ms,
        latest_block_height,
        reports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lwd_client::Network;

    #[test]
    fn creates_default_plan() {
        let endpoint = Endpoint::new(Network::Testnet, "https://example.invalid:9067");
        let plan = BenchmarkPlan::new(endpoint);

        assert_eq!(plan.requests, 10);
        assert_eq!(plan.timeout, Duration::from_secs(10));
    }

    #[test]
    fn summarizes_failed_reports() {
        let endpoint = Endpoint::new(Network::Testnet, "https://example.invalid:9067");
        let report = HealthReport {
            endpoint: endpoint.clone(),
            reachable: false,
            latest_block_height: None,
            estimated_block_height: None,
            vendor: None,
            version: None,
            consensus_branch_id: None,
            chain_name: None,
            latency_ms: 0,
            checked_at_unix: 0,
            message: "failed".to_string(),
        };

        let summary = summarize(endpoint.url, 1, vec![report]);

        assert_eq!(summary.successful_requests, 0);
        assert_eq!(summary.failed_requests, 1);
        assert_eq!(summary.average_ms, None);
    }
}
