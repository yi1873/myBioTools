use clap::{Parser, Subcommand};
use anyhow::Result;
use std::process;

use myBioTools::{select, split_fasta, tm, fasta_stats, hairpin, heterodimer, homodimer, rev_comp};

#[derive(Parser)]
#[command(
    name = "mybiotools",
    version = "0.1.0",
    about = "A collection of bioinformatics tools implemented in Rust",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract target lines/sequences or filter by length from FASTA
    Select(select::SelectArgs),
    /// Split multi-FASTA file into individual files
    SplitFasta(split_fasta::SplitFastaArgs),
    /// Calculate FASTA statistics (GC%, length, etc.)
    FastaStats(fasta_stats::FastaStatsArgs),
    /// Calculate hairpin formation ΔG (kcal/mol)
    Hairpin(hairpin::HairpinArgs),
    /// Calculate heterodimerization ΔG (kcal/mol)
    Heterodimer(heterodimer::HeterodimerArgs),
    /// Calculate homodimerization ΔG (kcal/mol)
    Homodimer(homodimer::HomodimerArgs),
    /// Generate reverse complement of DNA sequences
    RevComp(rev_comp::RevCompArgs),
    /// Calculate melting temperature (Tm) of DNA sequences
    Tm(tm::TmArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Select(args) => select::run(args),
        Commands::SplitFasta(args) => split_fasta::run(args),
        Commands::FastaStats(args) => fasta_stats::run(args),
        Commands::Hairpin(args) => hairpin::run(args),
        Commands::Heterodimer(args) => heterodimer::run(args),
        Commands::Homodimer(args) => homodimer::run(args),
        Commands::RevComp(args) => rev_comp::run(args),
        Commands::Tm(args) => tm::run(args),
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
