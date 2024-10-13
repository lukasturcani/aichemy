use clap::Parser;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    /// The URL of the Nomad server
    #[arg(value_parser=Url::from_str)]
    url: Url,

    /// Admin password
    #[arg(default_value = "foo")]
    admin_password: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let token = token(&client, &cli.url, &cli.admin_password).await?;
    let instruments = add_instruments(&client, &cli.url, &token).await?;
    let parameter_sets = add_parameter_sets(&client, &cli.url, &token, &instruments).await?;
    let groups = add_groups(&client, &cli.url, &token, &parameter_sets).await?;
    Ok(())
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct InstrumentId(String);

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Instrument {
    name: String,
    model: String,
    capacity: u8,
}

async fn add_instruments(
    client: &reqwest::Client,
    url: &Url,
    token: &str,
) -> Result<Vec<InstrumentId>, anyhow::Error> {
    let instruments = [
        Instrument {
            name: "Instrument 1".to_string(),
            model: "Model 1".to_string(),
            capacity: 1,
        },
        Instrument {
            name: "Instrument 2".to_string(),
            model: "Model 2".to_string(),
            capacity: 2,
        },
        Instrument {
            name: "Instrument 3".to_string(),
            model: "Model 3".to_string(),
            capacity: 3,
        },
    ];

    for instrument in instruments {
        client
            .post(url.join("/api/admin/instruments")?)
            .json(&instrument)
            .bearer_auth(token)
            .send()
            .await?;
    }

    Ok(())
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct ParameterSetId(String);

#[derive(Serialize, Debug, PartialEq, Eq)]
struct ParameterSet {
    name: String,
    description: String,
    #[serde(rename = "availableOn")]
    available_on: Vec<InstrumentId>,
}

async fn add_parameter_sets(
    client: &reqwest::Client,
    url: &Url,
    token: &str,
    instruments: &[InstrumentId],
) -> Result<Vec<ParameterSetId>, anyhow::Error> {
    todo!()
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct GroupId(String);

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Group {
    #[serde(rename = "groupName")]
    name: String,
    description: String,
    #[serde(rename = "isBatch")]
    is_batch: bool,
    #[serde(rename = "dataAccess")]
    data_access: String,
    #[serde(rename = "expList")]
    experiments: Vec<String>,
}

async fn add_groups(
    client: &reqwest::Client,
    url: &Url,
    token: &str,
    parameter_sets: &[ParameterSetId],
) -> Result<Vec<GroupId>, anyhow::Error> {
    todo!()
}

#[derive(Deserialize, Debug)]
struct AuthResponse {
    token: String,
}

async fn token(
    client: &reqwest::Client,
    url: &Url,
    password: &str,
) -> Result<String, anyhow::Error> {
    let response = client
        .post(url.join("/api/auth/login")?)
        .json(&json!({
            "username": "admin",
            "password": password,
        }))
        .send()
        .await?
        .json::<AuthResponse>()
        .await?;
    Ok(response.token)
}
