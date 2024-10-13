use aichemy::nmr::nomad_nmr::{Client, ExperimentQuery};

#[test]
fn download_all() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::login(
        std::env::var("NOMAD_URL").unwrap_or("http://localhost:8080".into()),
        "admin",
        "foo",
    )?;
    let experiments = client.experiments(ExperimentQuery::default())?;
    std::fs::write("/home/lt912/experiments.zip", experiments.get()?)?;
    Ok(())
}
