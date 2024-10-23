use std::{error::Error, fs, path::PathBuf};

use aichemy::nmr::io::jcamp_dx;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Path to the JCAMP-DX file
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let content = fs::read(cli.file)?;
    let records = jcamp_dx::parse(&content)?;
    println!("{:#?}", records);
    Ok(())
}
