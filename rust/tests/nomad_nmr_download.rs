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

    let experiments = client.auto_experiments(&AutoExperimentQuery::empty())?;

    let mut file = tempfile::tempfile()?;
    file.write_all(experiments.get()?.as_ref())?;
    file.seek(SeekFrom::Start(0))?;

    let expected_files = {
        let mut files = vec![
            "2106231050-2-1-test1-10.json",
            "2106231050-2-1-test1-11.json",
            "2106231055-3-2-test2-10.json",
            "2106231100-10-2-test3-10.json",
            "2106240012-10-2-test2-10.json",
            "2106241100-10-2-test3-10.json",
            "2106241100-10-2-test4-1.json",
        ];
        files.sort();
        files
    };

    let mut zip = ZipArchive::new(file)?;
    let actual_file_names = {
        let mut files: Vec<_> = zip.file_names().collect();
        files.sort();
        files
    };

    assert_eq!(expected_files, actual_file_names);

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        let stem = file.name().strip_suffix(".json").unwrap();
        assert_eq!(s, format!(r#""{stem}""#));
    }
    Ok(())
}
