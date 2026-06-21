use std::fmt;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rpc::compact_tx_streamer_client::CompactTxStreamerClient;
use rpc::Empty;
use serde::{Deserialize, Serialize};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint as TonicEndpoint};
use tonic::{Request, Status};

pub mod rpc {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Endpoint {
    pub network: Network,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointRecord {
    pub id: String,
    pub url: String,
    pub region: String,
    pub operator: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub status: EndpointStatus,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointRegistry {
    pub network: Network,
    pub updated_at: String,
    #[serde(default)]
    pub endpoints: Vec<EndpointRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Testnet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EndpointStatus {
    Active,
    Maintenance,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub endpoint: Endpoint,
    pub reachable: bool,
    pub latest_block_height: Option<u64>,
    pub estimated_block_height: Option<u64>,
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub consensus_branch_id: Option<String>,
    pub chain_name: Option<String>,
    pub latency_ms: u128,
    pub checked_at_unix: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeOptions {
    pub timeout: Duration,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
        }
    }
}

impl Endpoint {
    pub fn new(network: Network, url: impl Into<String>) -> Self {
        Self {
            network,
            url: url.into(),
        }
    }

    pub fn validate(&self) -> Result<(), EndpointError> {
        validate_endpoint_url(&self.url)
    }
}

impl EndpointRegistry {
    pub fn from_yaml(input: &str) -> Result<Self, RegistryError> {
        let registry: EndpointRegistry =
            serde_yaml::from_str(input).map_err(RegistryError::Yaml)?;
        registry.validate()?;
        Ok(registry)
    }

    pub fn validate(&self) -> Result<(), RegistryError> {
        if self.updated_at.trim().is_empty() {
            return Err(RegistryError::MissingField("updated_at"));
        }

        let mut ids = std::collections::HashSet::new();
        for record in &self.endpoints {
            validate_record(record)?;
            if !ids.insert(record.id.as_str()) {
                return Err(RegistryError::DuplicateEndpointId(record.id.clone()));
            }
        }

        Ok(())
    }

    pub fn active_endpoints(&self) -> impl Iterator<Item = Endpoint> + '_ {
        self.endpoints
            .iter()
            .filter(|record| record.status == EndpointStatus::Active)
            .map(|record| Endpoint::new(self.network, record.url.clone()))
    }
}

#[derive(Debug)]
pub enum ProbeError {
    Endpoint(EndpointError),
    Transport(tonic::transport::Error),
    Status(Status),
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointError {
    EmptyUrl,
    UnsupportedScheme,
}

#[derive(Debug)]
pub enum RegistryError {
    Yaml(serde_yaml::Error),
    MissingField(&'static str),
    DuplicateEndpointId(String),
    InvalidEndpointId(String),
    InvalidEndpointUrl { id: String, source: EndpointError },
    InvalidRecord { id: String, field: &'static str },
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeError::Endpoint(err) => write!(f, "{err}"),
            ProbeError::Transport(err) => write!(f, "transport error: {err}"),
            ProbeError::Status(err) => write!(f, "gRPC status error: {err}"),
            ProbeError::Timeout => write!(f, "probe timed out"),
        }
    }
}

impl std::error::Error for ProbeError {}

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

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::Yaml(err) => write!(f, "invalid YAML: {err}"),
            RegistryError::MissingField(field) => write!(f, "missing required field: {field}"),
            RegistryError::DuplicateEndpointId(id) => write!(f, "duplicate endpoint id: {id}"),
            RegistryError::InvalidEndpointId(id) => write!(f, "invalid endpoint id: {id}"),
            RegistryError::InvalidEndpointUrl { id, source } => {
                write!(f, "invalid endpoint URL for {id}: {source}")
            }
            RegistryError::InvalidRecord { id, field } => {
                write!(f, "invalid endpoint record {id}: missing {field}")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

pub async fn probe_lightwalletd(
    endpoint: Endpoint,
    options: ProbeOptions,
) -> Result<HealthReport, ProbeError> {
    endpoint.validate().map_err(ProbeError::Endpoint)?;
    let started = Instant::now();
    let checked_at_unix = unix_time();
    let timeout = options.timeout;

    let info = tokio::time::timeout(timeout, async {
        let channel = connect(&endpoint.url)
            .await
            .map_err(ProbeError::Transport)?;
        let mut client = CompactTxStreamerClient::new(channel);
        client
            .get_lightd_info(Request::new(Empty {}))
            .await
            .map(|response| response.into_inner())
            .map_err(ProbeError::Status)
    })
    .await
    .map_err(|_| ProbeError::Timeout)??;

    Ok(HealthReport {
        endpoint,
        reachable: true,
        latest_block_height: Some(info.block_height),
        estimated_block_height: Some(info.estimated_height),
        vendor: non_empty(info.vendor),
        version: non_empty(info.version),
        consensus_branch_id: non_empty(info.consensus_branch_id),
        chain_name: non_empty(info.chain_name),
        latency_ms: started.elapsed().as_millis(),
        checked_at_unix,
        message: "ok".to_string(),
    })
}

pub async fn probe_or_report(endpoint: Endpoint, options: ProbeOptions) -> HealthReport {
    match probe_lightwalletd(endpoint.clone(), options).await {
        Ok(report) => report,
        Err(err) => HealthReport {
            endpoint,
            reachable: false,
            latest_block_height: None,
            estimated_block_height: None,
            vendor: None,
            version: None,
            consensus_branch_id: None,
            chain_name: None,
            latency_ms: 0,
            checked_at_unix: unix_time(),
            message: err.to_string(),
        },
    }
}

async fn connect(url: &str) -> Result<Channel, tonic::transport::Error> {
    let endpoint = TonicEndpoint::from_shared(url.to_string())?;
    if url.starts_with("https://") {
        endpoint
            .tls_config(ClientTlsConfig::new().with_webpki_roots())?
            .connect()
            .await
    } else {
        endpoint.connect().await
    }
}

fn validate_record(record: &EndpointRecord) -> Result<(), RegistryError> {
    if record.id.trim().is_empty()
        || !record
            .id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(RegistryError::InvalidEndpointId(record.id.clone()));
    }

    if record.region.trim().is_empty() {
        return Err(RegistryError::InvalidRecord {
            id: record.id.clone(),
            field: "region",
        });
    }

    if record.operator.trim().is_empty() {
        return Err(RegistryError::InvalidRecord {
            id: record.id.clone(),
            field: "operator",
        });
    }

    validate_endpoint_url(&record.url).map_err(|source| RegistryError::InvalidEndpointUrl {
        id: record.id.clone(),
        source,
    })
}

fn validate_endpoint_url(url: &str) -> Result<(), EndpointError> {
    if url.trim().is_empty() {
        return Err(EndpointError::EmptyUrl);
    }

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(EndpointError::UnsupportedScheme);
    }

    Ok(())
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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

    #[test]
    fn parses_valid_registry() {
        let registry = EndpointRegistry::from_yaml(
            r#"
network: testnet
updated_at: "2026-06-21T00:00:00Z"
endpoints:
  - id: example-testnet
    url: https://example.invalid:9067
    region: test
    operator: example
    capabilities: [grpc, tls]
    status: active
"#,
        )
        .expect("registry should parse");

        assert_eq!(registry.network, Network::Testnet);
        assert_eq!(registry.active_endpoints().count(), 1);
    }

    #[test]
    fn rejects_duplicate_registry_ids() {
        let err = EndpointRegistry::from_yaml(
            r#"
network: mainnet
updated_at: "2026-06-21T00:00:00Z"
endpoints:
  - id: duplicate
    url: https://one.example.invalid:9067
    region: test
    operator: example
    status: active
  - id: duplicate
    url: https://two.example.invalid:9067
    region: test
    operator: example
    status: active
"#,
        )
        .expect_err("duplicate ids should fail");

        assert!(matches!(err, RegistryError::DuplicateEndpointId(_)));
    }
}
