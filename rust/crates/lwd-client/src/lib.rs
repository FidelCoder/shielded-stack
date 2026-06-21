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
        validate_timestamp(&self.updated_at)?;

        let mut ids = std::collections::HashSet::new();
        let mut urls = std::collections::HashSet::new();
        for record in &self.endpoints {
            validate_record(record)?;
            if !ids.insert(record.id.as_str()) {
                return Err(RegistryError::DuplicateEndpointId(record.id.clone()));
            }
            if !urls.insert(record.url.as_str()) {
                return Err(RegistryError::DuplicateEndpointUrl(record.url.clone()));
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
    MissingHost,
    MissingPort,
    ContainsPath,
    ContainsQuery,
    ContainsFragment,
}

#[derive(Debug)]
pub enum RegistryError {
    Yaml(serde_yaml::Error),
    MissingField(&'static str),
    InvalidTimestamp(String),
    DuplicateEndpointId(String),
    DuplicateEndpointUrl(String),
    DuplicateCapability { id: String, capability: String },
    InvalidCapability { id: String, capability: String },
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
            EndpointError::MissingHost => write!(f, "endpoint URL must include a host"),
            EndpointError::MissingPort => write!(f, "endpoint URL must include an explicit port"),
            EndpointError::ContainsPath => write!(f, "endpoint URL must not include a path"),
            EndpointError::ContainsQuery => {
                write!(f, "endpoint URL must not include a query string")
            }
            EndpointError::ContainsFragment => {
                write!(f, "endpoint URL must not include a fragment")
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
            RegistryError::InvalidTimestamp(value) => {
                write!(
                    f,
                    "updated_at must be a UTC timestamp like 2026-06-21T00:00:00Z: {value}"
                )
            }
            RegistryError::DuplicateEndpointId(id) => write!(f, "duplicate endpoint id: {id}"),
            RegistryError::DuplicateEndpointUrl(url) => write!(f, "duplicate endpoint URL: {url}"),
            RegistryError::DuplicateCapability { id, capability } => {
                write!(f, "duplicate capability for {id}: {capability}")
            }
            RegistryError::InvalidCapability { id, capability } => {
                write!(f, "invalid capability for {id}: {capability}")
            }
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
        || record.id.starts_with('-')
        || record.id.ends_with('-')
        || record.id.contains("--")
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

    validate_capabilities(record)?;
    validate_endpoint_url(&record.url).map_err(|source| RegistryError::InvalidEndpointUrl {
        id: record.id.clone(),
        source,
    })
}

fn validate_capabilities(record: &EndpointRecord) -> Result<(), RegistryError> {
    if record.status == EndpointStatus::Active
        && !record.capabilities.iter().any(|cap| cap == "grpc")
    {
        return Err(RegistryError::InvalidRecord {
            id: record.id.clone(),
            field: "capabilities.grpc",
        });
    }

    let mut seen = std::collections::HashSet::new();
    for capability in &record.capabilities {
        if !matches!(capability.as_str(), "grpc" | "tls" | "tor") {
            return Err(RegistryError::InvalidCapability {
                id: record.id.clone(),
                capability: capability.clone(),
            });
        }

        if !seen.insert(capability.as_str()) {
            return Err(RegistryError::DuplicateCapability {
                id: record.id.clone(),
                capability: capability.clone(),
            });
        }
    }

    Ok(())
}

fn validate_endpoint_url(url: &str) -> Result<(), EndpointError> {
    if url.trim().is_empty() {
        return Err(EndpointError::EmptyUrl);
    }

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(EndpointError::UnsupportedScheme);
    }

    let parsed = url
        .parse::<http::Uri>()
        .map_err(|_| EndpointError::MissingHost)?;
    if parsed.host().is_none() {
        return Err(EndpointError::MissingHost);
    }
    if parsed.port_u16().is_none() {
        return Err(EndpointError::MissingPort);
    }
    if parsed.path() != "/" {
        return Err(EndpointError::ContainsPath);
    }
    if parsed.query().is_some() {
        return Err(EndpointError::ContainsQuery);
    }
    if url.contains('#') {
        return Err(EndpointError::ContainsFragment);
    }

    Ok(())
}

fn validate_timestamp(value: &str) -> Result<(), RegistryError> {
    if value.trim().is_empty() {
        return Err(RegistryError::MissingField("updated_at"));
    }

    let valid_shape = value.len() == 20
        && value.as_bytes()[4] == b'-'
        && value.as_bytes()[7] == b'-'
        && value.as_bytes()[10] == b'T'
        && value.as_bytes()[13] == b':'
        && value.as_bytes()[16] == b':'
        && value.as_bytes()[19] == b'Z'
        && value
            .chars()
            .enumerate()
            .all(|(index, ch)| matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || ch.is_ascii_digit());

    if !valid_shape {
        return Err(RegistryError::InvalidTimestamp(value.to_string()));
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
    fn rejects_endpoint_without_port() {
        let endpoint = Endpoint::new(Network::Mainnet, "https://example.invalid");
        assert_eq!(endpoint.validate(), Err(EndpointError::MissingPort));
    }

    #[test]
    fn rejects_endpoint_with_path() {
        let endpoint = Endpoint::new(Network::Mainnet, "https://example.invalid:9067/status");
        assert_eq!(endpoint.validate(), Err(EndpointError::ContainsPath));
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
    capabilities: [grpc, tls]
    status: active
  - id: duplicate
    url: https://two.example.invalid:9067
    region: test
    operator: example
    capabilities: [grpc, tls]
    status: active
"#,
        )
        .expect_err("duplicate ids should fail");

        assert!(matches!(err, RegistryError::DuplicateEndpointId(_)));
    }

    #[test]
    fn rejects_duplicate_registry_urls() {
        let err = EndpointRegistry::from_yaml(
            r#"
network: mainnet
updated_at: "2026-06-21T00:00:00Z"
endpoints:
  - id: one
    url: https://duplicate.example.invalid:9067
    region: test
    operator: example
    capabilities: [grpc, tls]
    status: active
  - id: two
    url: https://duplicate.example.invalid:9067
    region: test
    operator: example
    capabilities: [grpc, tls]
    status: active
"#,
        )
        .expect_err("duplicate urls should fail");

        assert!(matches!(err, RegistryError::DuplicateEndpointUrl(_)));
    }

    #[test]
    fn rejects_active_endpoint_without_grpc_capability() {
        let err = EndpointRegistry::from_yaml(
            r#"
network: mainnet
updated_at: "2026-06-21T00:00:00Z"
endpoints:
  - id: no-grpc
    url: https://example.invalid:9067
    region: test
    operator: example
    capabilities: [tls]
    status: active
"#,
        )
        .expect_err("active endpoints should declare grpc");

        assert!(matches!(err, RegistryError::InvalidRecord { .. }));
    }

    #[test]
    fn rejects_invalid_capability() {
        let err = EndpointRegistry::from_yaml(
            r#"
network: mainnet
updated_at: "2026-06-21T00:00:00Z"
endpoints:
  - id: bad-capability
    url: https://example.invalid:9067
    region: test
    operator: example
    capabilities: [grpc, websocket]
    status: active
"#,
        )
        .expect_err("invalid capabilities should fail");

        assert!(matches!(err, RegistryError::InvalidCapability { .. }));
    }

    #[test]
    fn rejects_invalid_timestamp_shape() {
        let err = EndpointRegistry::from_yaml(
            r#"
network: mainnet
updated_at: "2026-06-21"
endpoints: []
"#,
        )
        .expect_err("invalid timestamp should fail");

        assert!(matches!(err, RegistryError::InvalidTimestamp(_)));
    }
}
