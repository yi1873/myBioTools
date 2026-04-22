use anyhow::Result;
use clap::Args;
use indexmap::IndexMap;
use needletail::parse_fastx_file;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;
use serde::Serialize;

#[derive(Args, Debug)]
#[command(
    about = "Calculate FASTA statistics (GC%, length, etc.)",
    long_about = "
Calculate FASTA statistics including length, GC%, AT%, N% for each sequence.
"
)]
pub struct FastaStatsArgs {
    /// Input FASTA file
    #[arg(short, long, value_name = "FILE")]
    pub input: String,
    
    /// Output file (default: print to stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,
    
    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct SequenceStats {
    seq_id: String,
    length: usize,
    gc_percent: f64,
    at_percent: f64,
    n_percent: f64,
    gc_count: usize,
    at_count: usize,
    n_count: usize,
}

pub fn run(args: FastaStatsArgs) -> Result<()> {
    let sequences = load_fasta(&args.input)?;
    let stats: Vec<SequenceStats> = sequences.iter()
        .map(|(id, seq)| {
            let stats = calculate_seq_stats(seq);
            SequenceStats {
                seq_id: id.clone(),
                length: stats.length,
                gc_percent: stats.gc_percent(),
                at_percent: stats.at_percent(),
                n_percent: stats.n_percent(),
                gc_count: stats.gc_count,
                at_count: stats.at_count,
                n_count: stats.n_count,
            }
        })
        .collect();
    
    match args.output {
        Some(output_path) => {
            if args.json {
                write_json_to_file(&stats, &output_path)?;
            } else {
                write_table_to_file(&stats, &output_path)?;
            }
        }
        None => {
            if args.json {
                let json = serde_json::to_string_pretty(&stats)?;
                println!("{}", json);
            } else {
                print_table(&stats);
            }
        }
    }
    
    Ok(())
}

struct SeqStatsInternal {
    length: usize,
    gc_count: usize,
    at_count: usize,
    n_count: usize,
}

impl SeqStatsInternal {
    fn from_seq(seq: &[u8]) -> Self {
        let mut gc = 0;
        let mut at = 0;
        let mut n = 0;

        for base in seq.iter().map(|b| b.to_ascii_uppercase()) {
            match base {
                b'G' | b'C' => gc += 1,
                b'A' | b'T' => at += 1,
                b'N' => n += 1,
                _ => {}
            }
        }

        Self {
            length: seq.len(),
            gc_count: gc,
            at_count: at,
            n_count: n,
        }
    }

    fn gc_percent(&self) -> f64 {
        (self.gc_count as f64 / self.length.max(1) as f64) * 100.0
    }

    fn at_percent(&self) -> f64 {
        (self.at_count as f64 / self.length.max(1) as f64) * 100.0
    }

    fn n_percent(&self) -> f64 {
        (self.n_count as f64 / self.length.max(1) as f64) * 100.0
    }
}

fn load_fasta(path: &str) -> Result<IndexMap<String, Vec<u8>>> {
    let mut reader = parse_fastx_file(path).map_err(|e| anyhow::anyhow!("Error opening file: {}", e))?;
    let mut sequences = IndexMap::new();

    while let Some(record) = reader.next() {
        let seqrec = record.map_err(|e| anyhow::anyhow!("Error reading FASTA record: {}", e))?;
        let raw_id = std::str::from_utf8(seqrec.id()).map_err(|e| anyhow::anyhow!("Invalid UTF-8 in sequence ID: {}", e))?;
        let id = raw_id.split_whitespace().next().unwrap_or(raw_id).to_string();
        let seq = seqrec.seq().to_vec();
        sequences.insert(id, seq);
    }

    Ok(sequences)
}

fn calculate_seq_stats(seq: &[u8]) -> SeqStatsInternal {
    SeqStatsInternal::from_seq(seq)
}

fn write_table_to_file(stats: &[SequenceStats], output_path: &str) -> Result<()> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    writeln!(
        writer,
        "SeqID\tLength\tGC%\tAT%\tN%\tGC_count\tAT_count\tN_count"
    )?;

    for stat in stats {
        writeln!(
            writer,
            "{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{}\t{}\t{}",
            stat.seq_id,
            stat.length,
            stat.gc_percent,
            stat.at_percent,
            stat.n_percent,
            stat.gc_count,
            stat.at_count,
            stat.n_count
        )?;
    }
    
    Ok(())
}

fn write_json_to_file(stats: &[SequenceStats], output_path: &str) -> Result<()> {
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &stats)?;
    Ok(())
}

fn print_table(stats: &[SequenceStats]) {
    println!("SeqID\tLength\tGC%\tAT%\tN%\tGC_count\tAT_count\tN_count");
    for stat in stats {
        println!(
            "{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{}\t{}\t{}",
            stat.seq_id,
            stat.length,
            stat.gc_percent,
            stat.at_percent,
            stat.n_percent,
            stat.gc_count,
            stat.at_count,
            stat.n_count
        );
    }
}
