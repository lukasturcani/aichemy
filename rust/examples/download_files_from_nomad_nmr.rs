use std::{error::Error, fs, path::PathBuf};

use aichemy::nmr::nomad_nmr::{AutoExperimentQuery, Client};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// URL of the NOMAD server
    url: String,

    /// Username
    username: String,

    /// Password
    password: String,

    /// Download path
    download_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let client = Client::login(cli.url, cli.username, cli.password)?;
    let experiments = client.auto_experiments(AutoExperimentQuery::default())?;
    fs::write(cli.download_path, experiments.get()?)?;
    Ok(())
}
