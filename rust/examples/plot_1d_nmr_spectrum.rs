use aichemy::nmr::bruker::{self, Procs};
use aichemy::nmr::io::jcamp_dx;
use clap::Parser;
use plotly::common::Mode;
use plotly::{Layout, Plot, Scatter};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Cli {
    /// Path to a Bruker NMR spectrum directory.
    nmr_directory: PathBuf,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let procs = read_procs(&cli.nmr_directory)?;
    let data = read_spectrum(&cli.nmr_directory)?;
    let spectrum = bruker::read_binary(data, procs.data_type()?, procs.endianness()?)?;
    let xs = (0..spectrum.len()).collect();
    let trace = Scatter::new(xs, spectrum)
        .name("spectrum")
        .mode(Mode::Lines);
    let mut plot = Plot::new();
    plot.add_trace(trace);
    let layout = Layout::new().title(format!("NMR Spectrum {}", cli.nmr_directory.display()));
    plot.set_layout(layout);
    plot.show();
    Ok(())
}

fn read_procs(nmr_directory: impl AsRef<Path>) -> Result<Procs, Box<dyn std::error::Error>> {
    let procs_path = glob::glob(
        nmr_directory
            .as_ref()
            .join("pdata/*/procs")
            .to_str()
            .ok_or(anyhow::anyhow!("invalid path"))?,
    )?
    .next()
    .ok_or(anyhow::anyhow!("no procs file found"))??;

    Ok(Procs(jcamp_dx::parse(fs::read(procs_path)?)?))
}

fn read_spectrum(nmr_directory: impl AsRef<Path>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let spectrum_path = glob::glob(
        nmr_directory
            .as_ref()
            .join("pdata/*/1r")
            .to_str()
            .ok_or(anyhow::anyhow!("invalid path"))?,
    )?
    .next()
    .ok_or(anyhow::anyhow!("no spectrum file found"))??;

    Ok(fs::read(spectrum_path)?)
}
