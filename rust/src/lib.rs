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

        use reqwest::{IntoUrl, Url};
        use serde::Deserialize;
        use serde_json::json;
        use std::path::Path;
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

        pub struct ExperimentQuery;
        pub struct DatasetQuery;

        #[derive(Debug, Deserialize)]
        struct AuthResponse {
            #[serde(rename = "expiresIn")]
            pub expires_in: String,
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
                let datetime = response
                    .headers()
                    .get("date")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let response: AuthResponse = response.json::<AuthResponse>().unwrap();
                println!("{:?}", datetime);
                Ok(Self {
                    client,
                    url,
                    username,
                    password,
                    auth_token: AuthToken {
                        token: response.token,
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
                let datetime = response
                    .headers()
                    .get("date")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                self.auth_token = AuthToken {
                    token: response.json::<AuthResponse>().unwrap().token,
                };
                Ok(self)
            }

            pub fn connect(url: Url) -> Self {
                todo!()
            }
            pub fn with_token(url: Url, token: impl AsRef<str>) -> Self {
                todo!()
            }

            pub fn experiments(&self, query: ExperimentQuery) -> Experiments {
                Experiments {
                    items: todo!(),
                    client: &self,
                }
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
