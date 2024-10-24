use aichemy::nmr::bruker::{self, Procs};
use aichemy::nmr::io::jcamp_dx;
use std::fs;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let procs = Procs(jcamp_dx::parse(fs::read("procs")?)?);
    let mut spectrum =
        bruker::read_binary(fs::read("1r")?, procs.data_type()?, procs.endianness()?)?;
    bruker::scale(&mut spectrum, procs.scale()?);
    Ok(())
}
