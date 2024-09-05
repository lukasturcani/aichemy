use aichemy::nmr::{self, NmrNomadClient};

#[test]
fn download() {
    let client = NmrNomadClient::connect("");
    for spectrum in client.spectra(Filter {}) {
        spectrum.download();
        let peaks = bruker::PickPicks::new().threshold(0.1).peaks(spectrum);
        let peaks = nmr::bruker::pick_peaks(path, nmr::bruker::PickPeakOptions {});
        db.insert(spectrum, peaks);
    }
    let paths = client.download_spectra(Filter {});
    nmr::bruker::pick_peaks();
    polars::read_database();
    let df = spectra.peaks();
    nmr::peaks(some_file);
    nmr::peaks(datasets.to_df().filter(col("users").eq("lukas")));
    train_model();
}
