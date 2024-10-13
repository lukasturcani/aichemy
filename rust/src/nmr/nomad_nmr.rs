use bytes::Bytes;
use chrono::{DateTime, Duration, Utc};
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to authenticate with NOMAD NMR server: {0}")]
    Auth(String),
    #[error("Failed to parse url: {source}")]
    InvalidUrl { source: reqwest::Error },
    #[error("Request failed: {source}")]
    Request { source: reqwest::Error },
}

#[derive(Debug, Clone)]
pub struct AuthToken {
    pub expiry_time: DateTime<Utc>,
    pub token: String,
}

impl AuthToken {
    pub fn expired(&self) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    pub inner: reqwest::blocking::Client,
    pub url: Url,
    pub username: String,
    pub password: String,
    pub auth_token: AuthToken,
}

#[derive(Debug, Clone, Default)]
pub struct AutoExperimentQuery {
    pub solvent: Vec<String>,
    pub instrument_id: Vec<String>,
    pub parameter_set: Vec<String>,
    pub title: Vec<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub group_id: Vec<String>,
    pub user_id: Vec<String>,
    pub dataset_name: Vec<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

impl AutoExperimentQuery {
    fn into_query(self) -> Vec<(String, String)> {
        let mut query = vec![];
        if !self.instrument_id.is_empty() {
            query.push(("instrumentId".to_string(), self.instrument_id.join(",")));
        }
        if !self.solvent.is_empty() {
            query.push(("solvent".to_string(), self.solvent.join(",")));
        }
        if !self.parameter_set.is_empty() {
            query.push(("paramSet".to_string(), self.parameter_set.join(",")));
        }
        if !self.title.is_empty() {
            query.push(("title".to_string(), self.title.join(",")));
        }
        if let Some(start_date) = self.start_date {
            query.push(("startDate".to_string(), start_date.to_rfc3339()));
        }
        if let Some(end_date) = self.end_date {
            query.push(("endDate".to_string(), end_date.to_rfc3339()));
        }
        if !self.group_id.is_empty() {
            query.push(("groupId".to_string(), self.group_id.join(",")));
        }
        if !self.user_id.is_empty() {
            query.push(("userId".to_string(), self.user_id.join(",")));
        }
        if !self.dataset_name.is_empty() {
            query.push(("datasetName".to_string(), self.dataset_name.join(",")));
        }
        if let Some(offset) = self.offset {
            query.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(limit) = self.limit {
            query.push(("limit".to_string(), limit.to_string()));
        }
        query
    }
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
    pub token: String,
}

fn deserialize_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    let s = s.map(|s| DateTime::parse_from_rfc3339(&s));
    match s {
        Some(Ok(dt)) => Ok(Some(dt.into())),
        Some(Err(source)) => Err(serde::de::Error::custom(source)),
        None => Ok(None),
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutoExperimentId(String);

#[derive(Debug, Deserialize, Clone)]
pub struct InstrumentId(String);

#[derive(Debug, Deserialize, Clone)]
pub struct UserId(String);

#[derive(Debug, Deserialize, Clone)]
pub struct GroupId(String);

#[derive(Debug, Deserialize, Clone)]
pub struct AutoExperiment {
    pub id: AutoExperimentId,

    #[serde(rename = "datasetName")]
    pub dataset_name: String,

    #[serde(rename = "expNo")]
    pub experiment_number: String,

    #[serde(rename = "parameterSet")]
    pub parameter_set: String,

    pub parameters: Option<String>,
    pub title: String,
    pub instrument: InstrumentId,
    pub user: UserId,
    pub group: GroupId,
    pub solvent: String,

    #[serde(
        default,
        rename = "submittedAt",
        deserialize_with = "deserialize_datetime"
    )]
    pub submitted_at: Option<DateTime<Utc>>,
}

impl Client {
    pub fn login(
        url: impl IntoUrl,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, Error> {
        let username = username.into();
        let password = password.into();
        let url = url
            .into_url()
            .map_err(|source| Error::InvalidUrl { source })?;
        let login_url = url.join("api/auth/login").unwrap();
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(login_url)
            .json(&json!({
                "username": username,
                "password": password,
            }))
            .send()
            .map_err(|source| Error::Request { source })?
            .error_for_status()
            .map_err(|source| Error::Request { source })?;
        let mut expiry_time = DateTime::parse_from_rfc2822(
            &response.headers().get("date").unwrap().to_str().unwrap()[5..],
        )
        .unwrap()
        .into();
        let response = response.json::<AuthResponse>().unwrap();
        expiry_time += Duration::seconds(response.expires_in);
        Ok(Self {
            inner: client,
            url,
            username,
            password,
            auth_token: AuthToken {
                token: response.token,
                expiry_time,
            },
        })
    }

    pub fn auth(&mut self) -> Result<&mut Self, Error> {
        let login_url = self.url.join("api/auth/login").unwrap();
        let response = self
            .inner
            .post(login_url)
            .json(&json!({
                "username": self.username,
                "password": self.password,
            }))
            .send()
            .map_err(|source| Error::Request { source })?
            .error_for_status()
            .map_err(|source| Error::Request { source })?;
        let mut expiry_time = DateTime::parse_from_rfc2822(
            &response.headers().get("date").unwrap().to_str().unwrap()[5..],
        )
        .unwrap()
        .into();
        let response = response.json::<AuthResponse>().unwrap();
        expiry_time += Duration::seconds(response.expires_in);
        self.auth_token = AuthToken {
            token: response.token,
            expiry_time,
        };
        Ok(self)
    }

    pub fn auto_experiments(&self, query: AutoExperimentQuery) -> Result<AutoExperiments, Error> {
        let response = self
            .inner
            .get(self.url.join("api/v2/auto-experiments").unwrap())
            .query(&query.into_query())
            .bearer_auth(self.auth_token.token.clone())
            .send()
            .map_err(|source| Error::Request { source })?
            .error_for_status()
            .map_err(|source| Error::Request { source })?
            .json::<Vec<AutoExperiment>>()
            .map_err(|source| Error::Request { source })?;
        Ok(AutoExperiments {
            inner: response,
            client: self,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AutoExperiments<'client> {
    pub inner: Vec<AutoExperiment>,
    pub client: &'client Client,
}

impl<'client> AutoExperiments<'client> {
    pub fn get(self) -> Result<Bytes, Error> {
        todo!()
        // self.client
        //     .inner
        //     .get(self.client.url.join("api/data/exps").unwrap())
        //     .query(&[
        //         (
        //             "exps",
        //             self.inner
        //                 .into_iter()
        //                 .flat_map(|experiment| experiment.runs.into_iter().map(|run| run.data.key))
        //                 .collect::<Vec<_>>()
        //                 .join(","),
        //         ),
        //         ("dataType", "auto".into()),
        //     ])
        //     .bearer_auth(self.client.auth_token.token.clone())
        //     .send()
        //     .map_err(|source| Error::Request { source })?
        //     .error_for_status()
        //     .map_err(|source| Error::Request { source })?
        //     .bytes()
        //     .map_err(|source| Error::Request { source })
    }
}
