use std::io::{Read, Seek, SeekFrom, Write};

use aichemy::nmr::nomad_nmr::{AutoExperimentQuery, Client};
use zip::ZipArchive;

#[test]
fn download_all() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::login(
        std::env::var("NOMAD_URL").unwrap_or("http://localhost:8080".into()),
        "admin",
        "foo",
    )?;
    let experiments = client.auto_experiments(AutoExperimentQuery::default())?;
    let mut file = tempfile::tempfile()?;
    file.write_all(experiments.get()?.as_ref())?;
    file.seek(SeekFrom::Start(0))?;
    let mut zip = ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let name = file.name();
        println!("{name}");
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        println!("{s}");
    }
    Ok(())
}
