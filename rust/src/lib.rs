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

    use reqwest::Url;
    use std::path::{Path, PathBuf};

    pub struct NmrNomadSpectra {
        pub storage_location: PathBuf,
        pub auth_token: PathBuf,
        pub force_download: bool,
        pub filter: Filter,
    }

    impl NmrNomadSpectra {
        pub fn download_from(&self, url: &Url, filter: Filter) {
            unimplemented!()
        }
    }

    impl Default for NmrNomadSpectra {
        fn default() -> Self {
            let storage_location = {
                let mut p = home::home_dir().unwrap();
                p.push(".aichemy");
                p.push("nmr_data");
                p
            };
            let auth_token = {
                let mut p = home::home_dir().unwrap();
                p.push(".nmr_nomad");
                p.push("auth_token");
                p
            };
            Self {
                storage_location,
                auth_token,
                force_download: false,
            }
        }
    }
}
