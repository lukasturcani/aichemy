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
        pub struct Client {
            client: reqwest::blocking::Client,
            pub url: Url,
            pub auth_token: String,
        }

        pub struct ExperimentQuery;
        pub struct DatasetQuery;

        #[derive(Deserialize)]
        struct AuthResponse {
            pub token: String,
        }

        impl Client {
            pub fn login(
                url: impl IntoUrl,
                username: impl AsRef<str>,
                password: impl AsRef<str>,
            ) -> Result<Self, Error> {
                let url = url
                    .into_url()
                    .map_err(|source| Error::InvalidUrl { source })?;
                let login_url = url.join("auth/login").unwrap();
                let client = reqwest::blocking::Client::new();
                let auth_token = client
                    .post(login_url)
                    .json(&json!({
                        "username": username.as_ref(),
                        "password": password.as_ref(),
                    }))
                    .send()
                    .map_err(|source| Error::Request { source })?
                    .error_for_status()
                    .map_err(|source| Error::Request { source })?
                    .json::<AuthResponse>()
                    .unwrap()
                    .token;
                Ok(Self {
                    client,
                    url,
                    auth_token,
                })
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
            pub items: Vec<Experiment>,
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
