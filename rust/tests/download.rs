use aichemy::nmr::nomad_nmr::{Client, ExperimentQuery};

#[test]
fn download() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::login(
        "http://aichemy-nmr.ch.ic.ac.uk",
        "admin",
        std::env::var("NOMAD_NMR_PASS").unwrap(),
    )?;
    let experiments = client.experiments(ExperimentQuery::default())?;
    let result = experiments.get()?;
    std::fs::write("/home/lt912/experiments.zip", result)?;
    Ok(())
    // expriments.download_stream().write();
    // let paths = client.download_spectra(Filter {});
    // nmr::bruker::pick_peaks();
    // polars::read_database();
    // let df = spectra.peaks();
    // nmr::peaks(some_file);
    // nmr::peaks(datasets.to_df().filter(col("users").eq("lukas")));
    // train_model();
}
