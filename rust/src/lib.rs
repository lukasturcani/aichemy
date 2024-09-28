// #![warn(rust_2018_idioms, missing_debug_implementations, missing_docs)]

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub mod nmr {

    pub mod nomad_nmr {

        use chrono::{DateTime, Duration, NaiveDate, Utc};
        use reqwest::{IntoUrl, Url};
        use serde::Deserialize;
        use serde_json::json;
        use std::{borrow::Borrow, path::Path};
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
            client: reqwest::blocking::Client,
            pub url: Url,
            pub username: String,
            pub password: String,
            pub auth_token: AuthToken,
        }

        #[derive(Debug, Clone)]
        pub struct DateRange {
            pub start: NaiveDate,
            pub end: NaiveDate,
        }

        impl DateRange {
            fn to_query(&self) -> String {
                format!(
                    "{},{}",
                    self.start.format("%Y-%m-%d"),
                    self.end.format("%Y-%m-%d")
                )
            }
        }

        #[derive(Debug, Clone, Default)]
        pub struct ExperimentQuery {
            pub instrument_id: Option<String>,
            pub solvent: Option<String>,
            pub parameter_set: Option<String>,
            pub title: Option<String>,
            pub date_range: Option<DateRange>,
            pub group_id: Option<String>,
            pub user_id: Option<String>,
            pub dataset_name: Option<String>,
            pub legacy_data: Option<bool>,
        }

        impl ExperimentQuery {
            fn to_query(&self) -> Vec<(String, String)> {
                let mut query = vec![("dataType".to_string(), "auto".to_string())];
                if let Some(instrument_id) = &self.instrument_id {
                    query.push(("instrumentId".to_string(), instrument_id.clone()));
                }
                if let Some(solvent) = &self.solvent {
                    query.push(("solvent".to_string(), solvent.clone()));
                }
                if let Some(parameter_set) = &self.parameter_set {
                    query.push(("paramSet".to_string(), parameter_set.clone()));
                }
                if let Some(title) = &self.title {
                    query.push(("title".to_string(), title.clone()));
                }
                if let Some(date_range) = &self.date_range {
                    query.push(("dateRange".to_string(), date_range.to_query()));
                }
                if let Some(group_id) = &self.group_id {
                    query.push(("groupId".to_string(), group_id.clone()));
                }
                if let Some(user_id) = &self.user_id {
                    query.push(("userId".to_string(), user_id.clone()));
                }
                if let Some(dataset_name) = &self.dataset_name {
                    query.push(("datasetName".to_string(), dataset_name.clone()));
                }
                if let Some(legacy_data) = &self.legacy_data {
                    query.push(("legacyData".to_string(), legacy_data.to_string()));
                }
                query
            }
        }
        pub struct DatasetQuery;

        #[derive(Debug, Deserialize)]
        struct AuthResponse {
            #[serde(rename = "expiresIn")]
            pub expires_in: i64,
            pub token: String,
        }

        #[derive(Debug, Deserialize)]
        struct ExperimentSearchResponse {
            data: Vec<ExperimentData>,
            total: usize,
            truncated: bool,
        }

        #[derive(Debug, Deserialize, Clone)]
        pub struct InstrumentData {
            pub id: String,
            pub name: String,
        }

        #[derive(Debug, Deserialize, Clone)]
        pub struct UserData {
            pub id: String,
            pub username: String,
        }

        #[derive(Debug, Deserialize, Clone)]
        pub struct GroupData {
            pub id: String,
            pub name: String,
        }

        #[derive(Debug, Deserialize, Clone)]
        pub struct ExperimentData {
            pub key: String,
            pub solvent: String,
            pub instrument: InstrumentData,
            pub user: UserData,
            pub group: GroupData,
            pub title: String,

            #[serde(rename = "datasetName")]
            pub dataset_name: String,
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
                let login_url = url.join("auth/login").unwrap();
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
                    client,
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
                let login_url = self.url.join("auth/login").unwrap();
                let response = self
                    .client
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

            pub fn connect(url: Url) -> Self {
                todo!()
            }
            pub fn with_token(url: Url, token: impl AsRef<str>) -> Self {
                todo!()
            }

            pub fn experiments(
                &self,
                query: impl Borrow<ExperimentQuery>,
            ) -> Result<Experiments, Error> {
                let query = query.borrow();
                let response = self
                    .client
                    .get(self.url.join("search/experiments").unwrap())
                    .query(&query.to_query())
                    .bearer_auth(self.auth_token.token.clone())
                    .send()
                    .map_err(|source| Error::Request { source })?
                    .json::<ExperimentSearchResponse>()
                    .map_err(|source| Error::Request { source })?;
                Ok(Experiments {
                    inner: response
                        .data
                        .into_iter()
                        .map(|data| Experiment { data, client: self })
                        .collect(),
                    client: self,
                })
            }

            pub fn datasets(&self, query: DatasetQuery) -> Datasets {
                todo!()
            }
        }

        #[derive(Debug, Clone)]
        pub struct Experiments<'client> {
            pub inner: Vec<Experiment<'client>>,
            client: &'client Client,
        }
        pub struct Datasets(pub Vec<Dataset>);

        impl<'client> Experiments<'client> {
            pub fn get(&self) {}
        }

        #[derive(Debug, Clone)]
        pub struct Experiment<'client> {
            pub data: ExperimentData,
            client: &'client Client,
        }

        impl<'client> Experiment<'client> {
            pub fn download(&self, path: impl AsRef<Path>) {}
        }

        pub struct Dataset {}

        pub fn experiments_to_peak_df(experiments: Experiments, download_path: impl AsRef<Path>) {
            todo!()
        }
    }

    pub mod bruker {}

    pub fn pick_peaks() {}
}
