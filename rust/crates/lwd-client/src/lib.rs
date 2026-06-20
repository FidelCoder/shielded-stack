use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub network: Network,
    pub url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub endpoint: Endpoint,
    pub reachable: bool,
    pub latest_block_height: Option<u64>,
    pub message: String,
}

impl Endpoint {
    pub fn new(network: Network, url: impl Into<String>) -> Self {
        Self {
            network,
            url: url.into(),
        }
    }

    pub fn validate(&self) -> Result<(), EndpointError> {
        if self.url.trim().is_empty() {
            return Err(EndpointError::EmptyUrl);
        }

        if !(self.url.starts_with("http://") || self.url.starts_with("https://")) {
            return Err(EndpointError::UnsupportedScheme);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointError {
    EmptyUrl,
    UnsupportedScheme,
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndpointError::EmptyUrl => write!(f, "endpoint URL is empty"),
            EndpointError::UnsupportedScheme => {
                write!(f, "endpoint URL must start with http:// or https://")
            }
        }
    }
}

impl std::error::Error for EndpointError {}

pub fn build_pending_report(endpoint: Endpoint) -> Result<HealthReport, EndpointError> {
    endpoint.validate()?;

    Ok(HealthReport {
        endpoint,
        reachable: false,
        latest_block_height: None,
        message: "gRPC probe not configured yet".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_http_endpoint() {
        let endpoint = Endpoint::new(Network::Testnet, "https://example.invalid:9067");
        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn rejects_unsupported_scheme() {
        let endpoint = Endpoint::new(Network::Mainnet, "tcp://example.invalid:9067");
        assert_eq!(endpoint.validate(), Err(EndpointError::UnsupportedScheme));
    }
}
