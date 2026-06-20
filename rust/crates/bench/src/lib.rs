use std::time::Duration;

use lwd_client::Endpoint;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkSummary {
    pub attempted_requests: u32,
    pub successful_requests: u32,
    pub fastest_response: Option<Duration>,
    pub slowest_response: Option<Duration>,
}

pub fn empty_summary(plan: &BenchmarkPlan) -> BenchmarkSummary {
    BenchmarkSummary {
        attempted_requests: plan.requests,
        successful_requests: 0,
        fastest_response: None,
        slowest_response: None,
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
}
