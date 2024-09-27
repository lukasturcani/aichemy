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
        use serde::{Deserialize, Serialize, Serializer};
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
            start: NaiveDate,
            end: NaiveDate,
        }

        impl Serialize for DateRange {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let s = format!(
                    "{},{}",
                    self.start.format("%Y-%m-%d"),
                    self.end.format("%Y-%m-%d")
                );
                serializer.serialize_str(&s)
            }
        }

        fn is_false(b: &bool) -> bool {
            !b
        }

        #[derive(Debug, Clone, Default, Serialize)]
        pub struct ExperimentQuery {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub instrument_id: Option<String>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub solvent: Option<String>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub parameter_set: Option<String>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub title: Option<String>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub date_range: Option<DateRange>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub group_id: Option<String>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub user_id: Option<String>,

            #[serde(skip_serializing_if = "is_false")]
            pub manual: bool,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub pulse_program: Option<String>,

            #[serde(skip_serializing_if = "Option::is_none")]
            pub dataset_name: Option<String>,

            #[serde(skip_serializing_if = "is_false")]
            pub legacy_data: bool,
        }
        pub struct DatasetQuery;

        #[derive(Debug, Deserialize)]
        struct AuthResponse {
            #[serde(rename = "expiresIn")]
            pub expires_in: i64,
            pub token: String,
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
                let mut expiry_time = DateTime::parse_from_str(
                    &response.headers().get("date").unwrap().to_str().unwrap()[5..],
                    "%d %b %Y %T %Z",
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
                let mut expiry_time = DateTime::parse_from_str(
                    &response.headers().get("date").unwrap().to_str().unwrap()[5..],
                    "%d %b %Y %T %Z",
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
                    .get(self.url.join("api/search/experiments").unwrap())
                    .query(&query)
                    .send()
                    .map_err(|source| Error::Request { source })?;
                println!("{:?}", response);
                todo!()
            }

            pub fn datasets(&self, query: DatasetQuery) -> Datasets {
                todo!()
            }
        }

        pub struct Experiments<'client> {
            pub inner: Vec<Experiment>,
            client: &'client Client,
        }
        pub struct Datasets(pub Vec<Dataset>);

        impl<'client> Experiments<'client> {
            pub fn get(&self) {}
        }

        pub struct Experiment {}

        impl Experiment {
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
