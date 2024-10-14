use std::collections::HashMap;
use axum::response::Html;
use axum::{
    Extension,
    Json,
    response::{
        IntoResponse,
        Response
    }
};
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampSeconds};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};
use tracing::debug;

#[derive(Debug, Deserialize, PartialEq)]
struct Stratum {
    #[serde(rename = "hashrate_15m")]
    hash_rate_15m: u64,
    #[serde(rename = "hashrate_1h")]
    hash_rate_1h: u64,
    #[serde(rename = "hashrate_24h")]
    hash_rate_24h: u64,

    shares_found: u64,
    shares_failed: u64,
    connections: u64,
    incoming_connections: u64,
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct NetworkStats {
    #[serde_as(as = "TimestampSeconds<i64>")]
    timestamp: SystemTime,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Metric<T> {
    name: String,
    definition: String,
    help: String,
    observation: Observation<T>,
}

impl<T> Metric<T> {
    pub fn new(name: String, definition: String, help: String, observation: Observation<T>) -> Self {
        Self { name, definition, help, observation }
    }
}

impl<T> fmt::Display for Metric<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = &self.name;
        let help = &self.help;
        let definition = &self.definition;
        let observation = &self.observation;
        write!(f, "# HELP {name} {help}\n# TYPE {name} {definition}\n{observation}")
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Observation<T> {
    #[serde(skip_serializing)]
    name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<HashMap<String, T>>,
}

impl<T> Observation<T> {
    pub fn new(name: String, label: Option<String>, value: Option<T>, values: Option<HashMap<String, T>>) -> Self {
        Self { name, label, value, values }
    }
}

impl<T> fmt::Display for Observation<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = &self.name;
        let serialised_self = match &self.value {
            None => match (&self.label, &self.values) {
                (Some(label), Some(values)) => {
                    let serialized_values = values
                        .iter()
                        .map(|(key, value)| format!(r#"{name}{{{label}="{key}"}} {value}"#))
                        .collect::<Vec<String>>()
                        .join("\n");
                    format!("{serialized_values}")
                },
                _ => panic!("failed to format observation for display"),
            },
            Some(value) => format!("{name} {value}")
        };
        write!(f, "{serialised_self}")
    }
}

///read file and return String
async fn get_file_str(file_path: PathBuf) -> String {
    debug!("reading file {}", file_path.display());
    let mut file = File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}

/// Reads the stratum file contents and returns Stratum struct
async fn get_stratum(stratum_str: String) -> Stratum {
    serde_json::from_str(&stratum_str).unwrap()
}

/// Reads the network stats contents and returns timestamp String
async fn get_network_timestamp(network_str: String) -> NetworkStats {
    let network_stats: NetworkStats = serde_json::from_str(&network_str).unwrap();
    network_stats
}

/// Converts timestamp EPOCH String to DateTime String
async fn convert_to_network_timestamp(network_str: String) -> String {
    let network_stats: NetworkStats = get_network_timestamp(network_str).await;

    <std::time::SystemTime as Into<DateTime<Utc>>>::into(network_stats.timestamp).to_string()
}

async fn get_prometheus_metrics(data_dir: PathBuf) -> Vec<Metric<u64>> {
    let stratum = get_stratum(get_file_str(data_dir.join("local").join("stratum")).await).await;
    let network = get_network_timestamp(get_file_str(data_dir.join("network").join("stats")).await).await;
    vec![
        Metric::new(
            "network_timestamp".to_string(),
            "gauge".to_string(),
            "network timestamp as seconds since unix epoch.".to_string(),
            Observation::new(
                "network_timestamp".to_string(),
                None,
                Some(network.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs()),
                None,
            ),
        ),
        Metric::new(
            "stratum_hash_rate".to_string(),
            "summary".to_string(),
            "a summary of the hash rate observed within an observation period.".to_string(),
            Observation::new(
                "stratum_hash_rate".to_string(),
                Some("period".to_string()),
                None,
                Some(HashMap::from([
                    ("15m".to_string(), stratum.hash_rate_15m),
                    ("1h".to_string(), stratum.hash_rate_1h),
                    ("24h".to_string(), stratum.hash_rate_24h),
                ])),
            ),
        ),
        Metric::new(
            "stratum_shares_found".to_string(),
            "gauge".to_string(),
            "number of found shares.".to_string(),
            Observation {
                name: "stratum_shares_found".to_string(),
                label: None,
                values: None,
                value: Some(stratum.shares_found),
            },
        ),
        Metric::new(
            "stratum_shares_failed".to_string(),
            "gauge".to_string(),
            "number of failed shares.".to_string(),
            Observation {
                name: "stratum_shares_failed".to_string(),
                label: None,
                values: None,
                value: Some(stratum.shares_failed),
            },
        ),
        Metric::new(
            "stratum_connections_outbound".to_string(),
            "gauge".to_string(),
            "number of outbound connections.".to_string(),
            Observation {
                name: "stratum_connections_outbound".to_string(),
                label: None,
                values: None,
                value: Some(stratum.connections),
            },
        ),
        Metric::new(
            "stratum_connections_inbound".to_string(),
            "gauge".to_string(),
            "number of inbound connections.".to_string(),
            Observation {
                name: "stratum_connections_inbound".to_string(),
                label: None,
                values: None,
                value: Some(stratum.incoming_connections),
            },
        ),
    ]
}


pub async fn serve_json_metrics(Extension(data_dir): Extension<PathBuf>) -> Response {
    Json(get_prometheus_metrics(data_dir).await).into_response()
}

pub async fn serve_prometheus_metrics(Extension(data_dir): Extension<PathBuf>) -> String {
    let metrics = get_prometheus_metrics(data_dir)
        .await
        .iter()
        .map(|metric| format!("{metric}"))
        .collect::<Vec<String>>()
        .join("\n");
    format!("{metrics}")
}

/// Populates HTML table with stratum JSON
pub async fn serve_stratum_table(Extension(data_dir): Extension<PathBuf>) -> Html<String> {
    let page_title = "Local Monero P2Pool stratum";

    let stratum_path = data_dir.join("local").join("stratum");
    let stratum_str = get_file_str(stratum_path).await;
    let stratum = get_stratum(stratum_str).await;
    let hash_rate_15m = stratum.hash_rate_15m;
    let hash_rate_1h = stratum.hash_rate_1h;
    let hash_rate_24h = stratum.hash_rate_24h;
    let shares_found = stratum.shares_found;
    let shares_failed = stratum.shares_failed;
    let connections = stratum.connections;
    let incoming_connections = stratum.incoming_connections;

    // read network file
    let network_path = data_dir.join("network").join("stats");
    let network_str = get_file_str(network_path).await;
    let timestamp = convert_to_network_timestamp(network_str).await;

    let html_table = format!(
        r#"<html lang="en">

    <head>
            <title>Monero P2Pool stats</title>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <link rel="stylesheet" href="https://maxcdn.bootstrapcdn.com/bootstrap/4.5.0/css/bootstrap.min.css">
            <script src="https://maxcdn.bootstrapcdn.com/bootstrap/4.5.0/js/bootstrap.min.js"></script>
        </head>
        <div class="container-fluid">
            <div class="row">
                <div class="col-md-12">
                    <h1>{page_title}</h1> <h2> {timestamp}</h2>
                </div>

            </div>

            <div class="row">
                <div class="col-md-6">
                    <table class="table table-striped">
                        <thead>
                            <tr>
                                <th scope="col">Hashrate [KH/s]</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>15m</td>
                                <td>{hash_rate_15m}</td>
                            </tr>
                            <tr>
                                <td>1h</td>
                                <td>{hash_rate_1h}</td>
                            </tr>
                            <tr>
                                <td>24h</td>
                                <td>{hash_rate_24h}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="row">
                <div class="col-md-6">
                    <table class="table table-striped">
                        <thead>
                            <tr>
                                <th scope="col">Shares [blocks]</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>found</td>
                                <td>{shares_found}</td>
                            </tr>
                            <tr>
                                <td>failed</td>
                                <td>{shares_failed}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <div class="row">
                <div class="col-md-6">
                    <table class="table table-striped">
                        <thead>
                            <tr>
                                <th scope="col">Connections</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>Outgoing</td>
                                <td>{connections}</td>
                            </tr>
                            <tr>
                                <td>Incoming</td>
                                <td>{incoming_connections}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

        </html>"#,
        page_title = page_title,
        hash_rate_15m = hash_rate_15m,
        hash_rate_1h = hash_rate_1h,
        hash_rate_24h = hash_rate_24h,
        shares_found = shares_found,
        shares_failed = shares_failed,
        connections = connections,
        incoming_connections = incoming_connections,
        timestamp = timestamp
    );
    Html(html_table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_network_timestamp() {
        let network_str = r#"{
            "difficulty": 326180875193,
            "hash": "5cc9cc40404608a866c16f4114a396355b82f8148c4285a21cd0937e8b84e776",
            "height": 2870723,
            "reward": 605959900000,
            "timestamp": 1682270152
        }"#;
        let ts = convert_to_network_timestamp(network_str.to_string()).await;
        assert_eq!(ts, "2023-04-23 17:15:52 UTC");
    }
    #[tokio::test]
    async fn test_get_stratum() {
        let network_str = r#"{
            "hashrate_15m": 10505,
            "hashrate_1h": 13794,
            "hashrate_24h": 24049,
            "total_hashes": 6021562332,
            "shares_found": 18,
            "shares_failed": 1,
            "average_effort": 122.298,
            "current_effort": 108.724,
            "connections": 2,
            "incoming_connections": 1
        }"#;
        let stratum = get_stratum(network_str.to_string()).await;
        assert_eq!(
            stratum,
            Stratum {
                hash_rate_1h: 13794000,
                hash_rate_15m: 10505000,
                hash_rate_24h: 24049000,
                shares_found: 18,
                shares_failed: 1,
                connections: 2,
                incoming_connections: 1
            }
        );
    }
}
