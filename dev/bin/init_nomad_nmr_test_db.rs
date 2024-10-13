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
    let users = add_users(&client, &cli.url, &token, &groups).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct InstrumentId(String);

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Instrument {
    name: String,
    model: String,
    capacity: u8,
}

#[derive(Deserialize, Debug)]
struct InstrumentResponse {
    #[serde(rename = "_id")]
    id: InstrumentId,
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

    let mut instrument_ids = Vec::with_capacity(instruments.len());
    for instrument in instruments {
        let instrument_id = client
            .post(url.join("/api/admin/instruments")?)
            .json(&instrument)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json::<InstrumentResponse>()
            .await?
            .id;
        instrument_ids.push(instrument_id);
    }

    Ok(instrument_ids)
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
struct ParameterSetId(String);

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
struct Param {
    name: String,
    value: Option<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct ParameterSet {
    name: String,
    description: String,
    #[serde(rename = "availableOn")]
    available_on: Vec<InstrumentId>,
    #[serde(rename = "defaultParams")]
    default_params: Vec<Param>,
}

#[derive(Deserialize, Debug)]
struct ParameterSetResponse {
    #[serde(rename = "_id")]
    id: ParameterSetId,
}

async fn add_parameter_sets(
    client: &reqwest::Client,
    url: &Url,
    token: &str,
    instruments: &[InstrumentId],
) -> Result<Vec<ParameterSetId>, anyhow::Error> {
    let parameter_sets = [
        ParameterSet {
            name: "ParamSet 1".to_string(),
            description: "Description 1".to_string(),
            available_on: vec![instruments[0].clone()],
            default_params: vec![],
        },
        ParameterSet {
            name: "ParamSet 2".to_string(),
            description: "Description 2".to_string(),
            available_on: vec![instruments[1].clone()],
            default_params: vec![],
        },
        ParameterSet {
            name: "ParamSet 3".to_string(),
            description: "Description 3".to_string(),
            available_on: vec![instruments[0].clone(), instruments[2].clone()],
            default_params: vec![],
        },
    ];
    let mut parameter_set_ids = Vec::with_capacity(parameter_sets.len());
    for parameter_set in parameter_sets {
        let parameter_set_id = client
            .post(url.join("/api/admin/param-sets")?)
            .json(&parameter_set)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json::<ParameterSetResponse>()
            .await?
            .id;
        parameter_set_ids.push(parameter_set_id);
    }
    Ok(parameter_set_ids)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct GroupId(String);

#[derive(Serialize, Debug, PartialEq, Eq)]
struct Group {
    #[serde(rename = "groupName")]
    name: String,
    description: String,
    #[serde(rename = "isBatch")]
    is_batch: bool,
    #[serde(rename = "expList")]
    parameter_sets: Vec<ParameterSetId>,
}

#[derive(Deserialize, Debug)]
struct GroupResponse {
    #[serde(rename = "_id")]
    id: GroupId,
}

async fn add_groups(
    client: &reqwest::Client,
    url: &Url,
    token: &str,
    parameter_sets: &[ParameterSetId],
) -> Result<Vec<GroupId>, anyhow::Error> {
    let groups = [
        Group {
            name: "Group 1".to_string(),
            description: "Description 1".to_string(),
            is_batch: false,
            parameter_sets: vec![parameter_sets[0].clone()],
        },
        Group {
            name: "Group 2".to_string(),
            description: "Description 2".to_string(),
            is_batch: true,
            parameter_sets: vec![parameter_sets[1].clone()],
        },
        Group {
            name: "Group 3".to_string(),
            description: "Description 3".to_string(),
            is_batch: false,
            parameter_sets: vec![parameter_sets[0].clone(), parameter_sets[2].clone()],
        },
    ];
    let mut group_ids = Vec::with_capacity(groups.len());
    for group in groups {
        let group_id = client
            .post(url.join("/api/admin/groups")?)
            .json(&group)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json::<GroupResponse>()
            .await?
            .id;
        group_ids.push(group_id);
    }
    Ok(group_ids)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct UserId(String);

#[derive(Serialize, Debug, PartialEq, Eq)]
struct User {
    username: String,
    password: String,
    email: String,
    #[serde(rename = "fullName")]
    full_name: String,
    #[serde(rename = "groupId")]
    group_id: GroupId,
}

#[derive(Deserialize, Debug)]
struct UserResponse {
    #[serde(rename = "_id")]
    id: UserId,
}

async fn add_users(
    client: &reqwest::Client,
    url: &Url,
    token: &str,
    groups: &[GroupId],
) -> Result<Vec<UserId>, anyhow::Error> {
    let users = [
        User {
            username: "user1".to_string(),
            password: "password1".to_string(),
            email: "user1@example.com".to_string(),
            full_name: "User One".to_string(),
            group_id: groups[0].clone(),
        },
        User {
            username: "user2".to_string(),
            password: "password2".to_string(),
            email: "user2@example.com".to_string(),
            full_name: "User Two".to_string(),
            group_id: groups[1].clone(),
        },
        User {
            username: "user3".to_string(),
            password: "password3".to_string(),
            email: "user3@example.com".to_string(),
            full_name: "User Three".to_string(),
            group_id: groups[0].clone(),
        },
    ];
    let mut user_ids = Vec::with_capacity(users.len());
    for user in users {
        let user_id = client
            .post(url.join("/api/admin/users")?)
            .json(&user)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json::<UserResponse>()
            .await?
            .id;
        user_ids.push(user_id);
    }
    Ok(user_ids)
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
