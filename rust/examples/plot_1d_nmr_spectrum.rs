use aichemy::nmr::bruker::{self, Procs};
use aichemy::nmr::io::jcamp_dx;
use clap::Parser;
use find_peaks::{Peak, PeakFinder};
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

    let mut spectrum = bruker::read_binary(data, procs.data_type()?, procs.endianness()?)?;
    bruker::scale(&mut spectrum, procs.scale()?);
    let xs = (0..spectrum.len()).collect();
    let peaks = PeakFinder::new(&spectrum)
        .with_min_prominence(1e4)
        .find_peaks();

    let mut plot = plot(&format!("NMR Spectrum {}", cli.nmr_directory.display()));
    plot_spectrum(&mut plot, spectrum, xs);
    plot_peaks(&mut plot, peaks);

    plot.show();
    Ok(())
}

fn plot(title: &str) -> Plot {
    let mut plot = Plot::new();
    let layout = Layout::new().title(title);
    plot.set_layout(layout);
    plot
}

fn plot_spectrum(plot: &mut Plot, spectrum: Vec<f64>, xs: Vec<usize>) {
    let trace = Scatter::new(xs, spectrum)
        .name("spectrum")
        .mode(Mode::Lines);
    plot.add_trace(trace);
}

fn plot_peaks(plot: &mut Plot, peaks: Vec<Peak<f64>>) {
    let trace = Scatter::new(
        peaks.iter().map(|peak| peak.middle_position()).collect(),
        peaks.iter().map(|peak| peak.height).collect(),
    )
    .name("peaks")
    .mode(Mode::Markers);
    plot.add_trace(trace);
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
