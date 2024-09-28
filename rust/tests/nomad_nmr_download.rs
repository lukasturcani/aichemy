use aichemy::nmr::nomad_nmr::{Client, ExperimentQuery};

#[test]
fn download_all() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::login(
        "http://aichemy-nmr.ch.ic.ac.uk",
        "admin",
        std::env::var("NOMAD_NMR_PASS").unwrap(),
    )?;
    let experiments = client.experiments(ExperimentQuery::default())?;
    std::fs::write("/home/lt912/experiments.zip", experiments.get()?)?;
    Ok(())
}
