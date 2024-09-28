use aichemy::nmr::nomad_nmr::{Client, ExperimentQuery};

#[test]
fn download() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::login("http://localhost:8080", "admin", "12345")?;
    let experiments = client.experiments(ExperimentQuery::default())?;
    println!(
        "{:#?}",
        experiments
            .inner
            .into_iter()
            .map(|e| e.data)
            .collect::<Vec<_>>()
    );
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
