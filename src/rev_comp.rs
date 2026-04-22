use anyhow::{Result, anyhow};
use clap::Args;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Args, Debug)]
#[command(
    about = "Generate reverse complement of DNA sequences",
    long_about = "
Generate reverse complement of DNA sequences.

Supports two input modes:
  1. Direct sequence input via -i/--input
  2. FASTA file input via --input-file

For FASTA files, each sequence will be converted to its reverse complement,
preserving the original header information.
"
)]
pub struct RevCompArgs {
    /// Input DNA sequence (direct input)
    #[arg(short, long, value_name = "SEQ", conflicts_with = "input_file")]
    pub input: Option<String>,
    
    /// Input FASTA file
    #[arg(long = "input-file", value_name = "FILE", conflicts_with = "input")]
    pub input_file: Option<String>,
    
    /// Output file (for FASTA mode, default: stdout)
    #[arg(long = "output-file", value_name = "FILE")]
    pub output_file: Option<String>,
}

pub fn run(args: RevCompArgs) -> Result<()> {
    match (&args.input, &args.input_file) {
        (Some(seq), None) => {
            // Direct sequence input mode
            let revcomp = reverse_complement(seq)?;
            
            if let Some(output_path) = &args.output_file {
                let mut writer = BufWriter::new(File::create(output_path)?);
                write_sequence(&mut writer, &revcomp)?;
            } else {
                print_sequence(&revcomp);
            }
        }
        (None, Some(input_path)) => {
            // FASTA file input mode
            process_fasta_file(input_path, &args.output_file)?;
        }
        _ => {
            return Err(anyhow!(
                "Either --input (direct sequence) or --input-file (FASTA file) must be provided"
            ));
        }
    }
    
    Ok(())
}

/// Generate reverse complement of a DNA sequence
fn reverse_complement(seq: &str) -> Result<String> {
    let mut result = String::with_capacity(seq.len());
    
    for c in seq.chars().rev() {
        let complement = match c.to_ascii_uppercase() {
            'A' => 'T',
            'T' => 'A',
            'U' => 'A', // Treat U as T for RNA compatibility
            'C' => 'G',
            'G' => 'C',
            'R' => 'Y', // Purine (A/G) -> Pyrimidine (T/C)
            'Y' => 'R', // Pyrimidine (T/C) -> Purine (A/G)
            'M' => 'K', // Amino (A/C) -> Keto (T/G)
            'K' => 'M', // Keto (T/G) -> Amino (A/C)
            'S' => 'S', // Strong (C/G) -> Strong (C/G)
            'W' => 'W', // Weak (A/T) -> Weak (A/T)
            'H' => 'D', // Not G (A/T/C) -> Not C (A/T/G)
            'B' => 'V', // Not A (T/C/G) -> Not T (A/C/G)
            'V' => 'B', // Not T (A/C/G) -> Not A (T/C/G)
            'D' => 'H', // Not C (A/T/G) -> Not G (A/T/C)
            'N' => 'N', // Any base
            _ => return Err(anyhow!("Invalid DNA character: {}", c)),
        };
        
        // Preserve original case
        if c.is_lowercase() {
            result.push(complement.to_ascii_lowercase());
        } else {
            result.push(complement);
        }
    }
    
    Ok(result)
}

/// Process FASTA file and generate reverse complements
fn process_fasta_file(input_path: &str, output_path: &Option<String>) -> Result<()> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    
    // Determine output writer
    let writer: Box<dyn Write> = match output_path {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(BufWriter::new(io::stdout())),
    };
    let mut writer = writer;
    
    let mut current_id = String::new();
    let mut current_desc = String::new();
    let mut current_seq = String::new();
    let mut in_record = false;
    
    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with('>') {
            // Process previous record
            if in_record && !current_seq.is_empty() {
                let revcomp = reverse_complement(&current_seq)?;
                write_fasta_record(&mut writer, &current_id, &current_desc, &revcomp)?;
            }
            
            // Parse new header
            let header = line[1..].trim();
            let parts: Vec<&str> = header.splitn(2, char::is_whitespace).collect();
            current_id = parts[0].to_string();
            current_desc = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
            current_seq.clear();
            in_record = true;
        } else if in_record {
            current_seq.push_str(&line.trim());
        }
    }
    
    // Process last record
    if in_record && !current_seq.is_empty() {
        let revcomp = reverse_complement(&current_seq)?;
        write_fasta_record(&mut writer, &current_id, &current_desc, &revcomp)?;
    }
    
    Ok(())
}

/// Write FASTA record to writer
fn write_fasta_record<W: Write>(writer: &mut W, id: &str, desc: &str, seq: &str) -> Result<()> {
    if desc.is_empty() {
        writeln!(writer, ">{}_revcomp", id)?;
    } else {
        writeln!(writer, ">{}_revcomp {}", id, desc)?;
    }
    write_sequence(writer, seq)?;
    Ok(())
}

/// Write sequence in 80-character chunks
fn write_sequence<W: Write>(writer: &mut W, seq: &str) -> Result<()> {
    let chunk_size = 80;
    for chunk in seq.as_bytes().chunks(chunk_size) {
        writer.write_all(chunk)?;
        writeln!(writer)?;
    }
    Ok(())
}

/// Print sequence in 80-character chunks
fn print_sequence(seq: &str) {
    let chunk_size = 80;
    for chunk in seq.as_bytes().chunks(chunk_size) {
        println!("{}", String::from_utf8_lossy(chunk));
    }
}
