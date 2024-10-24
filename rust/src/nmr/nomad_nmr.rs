//! Tools to interact with NOMAD NMR.
//!
//! A [`NOMAD NMR`] deployment is used by NMR labs to manage their machines
//! and store their data in a central place and in a
//! [FAIR](https://en.wikipedia.org/wiki/FAIR_data) manner. It automatically
//! provides features such as a monitoring system and a data repository which includes
//! metadata and access control.
//!
//! The NOMAD NMR [server](https://github.com/nomad-nmr/nomad-server) provides a
//! REST API to interact with it, which this module relies upon. The primary goal of
//! this module is to provide an interface for downloading large datasets from the
//! NOMAD server and turn them into data frames which can be used for machine learning.
//!
//! # Examples
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use aichemy::nmr::nomad_nmr::{AutoExperimentQuery, Client};
//! use std::fs;
//!
//! let mut client = Client::login(
//!     "http://demo.nomad-nmr.uk",
//!     "demo", // username
//!     "dem0User", // password
//! )?;
//!
//! // Download auto experiments into a zip archive.
//! let experiments = client.auto_experiments(&AutoExperimentQuery::empty())?;
//! fs::write("experiments.zip", experiments.download()?)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`NOMAD NMR`]: https://www.nomad-nmr.uk

use bytes::Bytes;
use chrono::{DateTime, Duration, Utc};
use reqwest::{IntoUrl, Url};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use thiserror::Error;

/// Error which may occur when interacting with the NOMAD server.
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to authenticate with the NOMAD server.
    #[error("failed to authenticate with NOMAD NMR server: {0}")]
    Auth(String),
    /// Failed to parse the URL.
    #[error("failed to parse url")]
    InvalidUrl {
        /// The underlying error.
        source: reqwest::Error,
    },
    /// Request failed.
    #[error("{source}")]
    Request {
        /// The underlying error.
        source: reqwest::Error,
    },
}

/// Authentication token for the NOMAD server.
///
/// A token must be used to authenticate requests to the NOMAD server. Generally
/// produced by the [Client::login] and [Client::auth] methods.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AuthToken {
    /// Time at which the token expires.
    pub expiry_time: DateTime<Utc>,
    /// The value of the token.
    pub token: String,
}

impl AuthToken {
    /// Check if the token is expired.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = aichemy::nmr::nomad_nmr::Client {
    /// #     inner: reqwest::blocking::Client::new(),
    /// #     url: reqwest::Url::parse("https://example.com")?,
    /// #     username: "username".to_string(),
    /// #     password: "password".to_string(),
    /// #     auth_token: aichemy::nmr::nomad_nmr::AuthToken {
    /// #         token: "token".to_string(),
    /// #         expiry_time: chrono::Utc::now() + chrono::Duration::days(1),
    /// #     },
    /// # };
    /// // Generate a new token if the current one is expired.
    /// if client.auth_token.expired() {
    ///     client.auth()?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn expired(&self) -> bool {
        self.expiry_time < Utc::now()
    }
}

/// Client for interacting with the NOMAD server.
///
/// Use the methods on the client to send requests to the NOMAD server.
///
/// # Examples
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use aichemy::nmr::nomad_nmr::{AutoExperimentQuery, Client};
/// use std::fs;
///
/// let mut client = Client::login(
///     "http://demo.nomad-nmr.uk",
///     "demo", // username
///     "dem0User", // password
/// )?;
///
/// // Download auto experiments into a zip archive.
/// let experiments = client.auto_experiments(&AutoExperimentQuery::empty())?;
/// fs::write("experiments.zip", experiments.download()?)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    /// The underlying [reqwest::blocking::Client].
    pub inner: reqwest::blocking::Client,
    /// The URL of the NOMAD server.
    pub url: Url,
    /// The username to use for authentication.
    pub username: String,
    /// The password to use for authentication.
    pub password: String,
    /// The authentication token to use for requests.
    pub auth_token: AuthToken,
}

impl Client {
    /// Create a new client by logging into the NOMAD server.
    ///
    /// # Examples
    /// [See here.](Client#examples)
    ///
    /// # Errors
    /// This method will return an error if the URL is invalid or if the
    /// authentication request fails.
    ///
    /// # Panics
    /// This method will panic if the response from the server does not
    /// match the expected format.
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
        let response = response
            .json::<AuthResponse>()
            .expect("auth response does not match expected format");
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

    /// Make the client use a new authentication token.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = aichemy::nmr::nomad_nmr::Client {
    /// #     inner: reqwest::blocking::Client::new(),
    /// #     url: reqwest::Url::parse("https://example.com")?,
    /// #     username: "username".to_string(),
    /// #     password: "password".to_string(),
    /// #     auth_token: aichemy::nmr::nomad_nmr::AuthToken {
    /// #         token: "token".to_string(),
    /// #         expiry_time: chrono::Utc::now() + chrono::Duration::days(1),
    /// #     },
    /// # };
    /// // Generate a new token if the current one is expired.
    /// if client.auth_token.expired() {
    ///     client.auth()?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// This method will return an error if the authentication request fails.
    ///
    /// # Panics
    /// This method will panic if the response from the server does not
    /// match the expected format.
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
        let response = response
            .json::<AuthResponse>()
            .expect("auth response does not match expected format");
        expiry_time += Duration::seconds(response.expires_in);
        self.auth_token = AuthToken {
            token: response.token,
            expiry_time,
        };
        Ok(self)
    }

    /// Get a list of auto experiments.
    ///
    /// # Examples
    ///
    /// ## Get all auto experiments
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client: aichemy::nmr::nomad_nmr::Client = todo!();
    /// use aichemy::nmr::nomad_nmr::AutoExperimentQuery;
    /// let auto_experiments = client.auto_experiments(&AutoExperimentQuery::empty())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Get auto experiments matching a specific query
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client: aichemy::nmr::nomad_nmr::Client = todo!();
    /// use aichemy::nmr::nomad_nmr::AutoExperimentQuery;
    /// let auto_experiments = client.auto_experiments(&AutoExperimentQuery {
    ///     solvent: vec!["CDCl3"],
    ///     instrument_id: vec!["1", "2", "4"],
    ///     user_id: vec!["foo", "bar"],
    ///     ..Default::default()
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// This method will return an error if the request fails.
    ///
    /// # Panics
    /// This method will panic if the response from the server does not
    /// match the expected format.
    pub fn auto_experiments<T>(
        &self,
        query: &AutoExperimentQuery<T>,
    ) -> Result<AutoExperiments<'_>, Error>
    where
        T: AsRef<str>,
    {
        let response = self
            .inner
            .get(self.url.join("api/v2/auto-experiments").unwrap())
            .query(&query.to_query())
            .bearer_auth(self.auth_token.token.clone())
            .send()
            .map_err(|source| Error::Request { source })?
            .error_for_status()
            .map_err(|source| Error::Request { source })?
            .json::<Vec<AutoExperiment>>()
            .expect("auto experiments response does not match expected format");
        Ok(AutoExperiments {
            inner: response,
            client: self,
        })
    }
}

/// Query parameters for the [`Client::auto_experiments`] method.
///
/// Each `Vec` field is a list of values to match. If the field is empty, no
/// filtering is performed. If multiple values are provided in a field,
/// the query will match if any of the values match.
///
/// If multple fields are provided, both fields must match for the query to
/// match. For example:
/// ```
/// use aichemy::nmr::nomad_nmr::AutoExperimentQuery;
/// let query = AutoExperimentQuery {
///     solvent: vec!["water", "methanol"],
///     instrument_id: vec!["1", "2"],
///     ..Default::default()
/// };
/// ```
/// will match all auto experiments with a solvent of water or methanol AND
/// an instrument ID of 1 or 2.
///
/// # Examples
///
/// [See here.](Client::auto_experiments)
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct AutoExperimentQuery<T> {
    /// Filter for experiments with any of these solvents.
    pub solvent: Vec<T>,
    /// Filter for experiments done on any of these instruments.
    pub instrument_id: Vec<T>,
    /// Filter for experiments using any of these parameter sets.
    pub parameter_set: Vec<T>,
    /// Filter for experiments with any of these titles.
    pub title: Vec<T>,
    /// Filter for experiments submitted after this date.
    pub start_date: Option<DateTime<Utc>>,
    /// Filter for experiments submitted before this date.
    pub end_date: Option<DateTime<Utc>>,
    /// Filter for experiments belonging to any of these groups.
    pub group_id: Vec<T>,
    /// Filter for experiments created by any of these users.
    pub user_id: Vec<T>,
    /// Filter for experiments in any of these datasets.
    pub dataset_name: Vec<T>,
    /// Skip the first `offset` experiments.
    pub offset: Option<usize>,
    /// Limit the number of experiments returned to `limit`.
    pub limit: Option<usize>,
}

impl AutoExperimentQuery<String> {
    /// Create a new empty query.
    ///
    /// This query will match all experiments.
    pub fn empty() -> Self {
        Self::default()
    }
}

impl<T> AutoExperimentQuery<T>
where
    T: AsRef<str>,
{
    fn to_query(&self) -> Vec<(String, String)> {
        let mut query = vec![];
        if !self.instrument_id.is_empty() {
            query.push((
                "instrumentId".to_string(),
                self.instrument_id
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if !self.solvent.is_empty() {
            query.push((
                "solvent".to_string(),
                self.solvent
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if !self.parameter_set.is_empty() {
            query.push((
                "paramSet".to_string(),
                self.parameter_set
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if !self.title.is_empty() {
            query.push((
                "title".to_string(),
                self.title
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if let Some(start_date) = self.start_date {
            query.push(("startDate".to_string(), start_date.to_rfc3339()));
        }
        if let Some(end_date) = self.end_date {
            query.push(("endDate".to_string(), end_date.to_rfc3339()));
        }
        if !self.group_id.is_empty() {
            query.push((
                "groupId".to_string(),
                self.group_id
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if !self.user_id.is_empty() {
            query.push((
                "userId".to_string(),
                self.user_id
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        if !self.dataset_name.is_empty() {
            query.push((
                "datasetName".to_string(),
                self.dataset_name
                    .iter()
                    .map(|s| s.as_ref())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
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

/// A unique id for an auto experiment.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct AutoExperimentId(pub String);

/// A unique id for an instrument.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct InstrumentId(pub String);

/// A unique id for a NOMAD user.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct UserId(pub String);

/// A unique id for a group.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct GroupId(pub String);

/// Data about an auto experiment stored in NOMAD.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct AutoExperiment {
    /// The unique id of the auto experiment.
    pub id: AutoExperimentId,

    /// The name of the dataset the experiment belongs to.
    #[serde(rename = "datasetName")]
    pub dataset_name: String,

    /// The experiment number.
    #[serde(rename = "expNo")]
    pub experiment_number: String,

    /// The parameter set used to run the experiment.
    #[serde(rename = "parameterSet")]
    pub parameter_set: String,

    /// The parameters used to run the experiment.
    pub parameters: Option<String>,

    /// The title of the experiment.
    pub title: String,

    /// The instrument used to run the experiment.
    pub instrument: InstrumentId,

    /// The user who ran the experiment.
    pub user: UserId,

    /// The group the experiment belongs to.
    pub group: GroupId,

    /// The solvent used in the experiment.
    pub solvent: String,

    /// The date and time the experiment was submitted.
    #[serde(
        default,
        rename = "submittedAt",
        deserialize_with = "deserialize_datetime"
    )]
    pub submitted_at: Option<DateTime<Utc>>,
}

/// A collection of auto experiments stored in NOMAD.
///
/// Use this if you want to download the auto experiments as a zip archive.
///
/// # Examples
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut client: aichemy::nmr::nomad_nmr::Client = todo!();
/// use std::fs;
/// use aichemy::nmr::nomad_nmr::AutoExperimentQuery;
/// let auto_experiments = client.auto_experiments(&AutoExperimentQuery::empty())?;
/// fs::write("experiments.zip", auto_experiments.download()?)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AutoExperiments<'client> {
    /// The auto experiment data.
    pub inner: Vec<AutoExperiment>,
    /// The client which can be used to download the auto experiments.
    pub client: &'client Client,
}

impl<'client> AutoExperiments<'client> {
    /// Download the auto experiments as a zip archive.
    ///
    /// # Examples
    /// [See here.](AutoExperiments#examples)
    ///
    /// # Errors
    /// This method will return an error if the request fails.
    pub fn download(self) -> Result<Bytes, Error> {
        self.client
            .inner
            .post(
                self.client
                    .url
                    .join("api/v2/auto-experiments/download")
                    .unwrap(),
            )
            .query(&[(
                "id",
                self.inner
                    .into_iter()
                    .map(|experiment| experiment.id.0)
                    .collect::<Vec<_>>()
                    .join(","),
            )])
            .bearer_auth(self.client.auth_token.token.clone())
            .send()
            .map_err(|source| Error::Request { source })?
            .error_for_status()
            .map_err(|source| Error::Request { source })?
            .bytes()
            .map_err(|source| Error::Request { source })
    }
}
