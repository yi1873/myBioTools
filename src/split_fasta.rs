use anyhow::{Result, anyhow};
use clap::Args;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
#[command(
    about = "Split multi-FASTA file into individual files",
    long_about = "
Split multi-FASTA file into individual files

Each sequence will be written to a separate file named after the sequence ID.
The output directory can be specified with --output-dir (default: ./split_fasta.subdir).
"
)]
pub struct SplitFastaArgs {
    /// Input multi-FASTA file
    #[arg(short, long, value_name = "FILE")]
    pub multifasta: String,
    
    /// Output directory
    #[arg(short = 'd', long = "output-dir", default_value = "./split_fasta.subdir", value_name = "DIR")]
    pub output_dir: String,
    
    /// File extension for output files (without dot)
    #[arg(short = 'e', long = "extension", default_value = "fasta", value_name = "EXT")]
    pub extension: String,
}

pub fn run(args: SplitFastaArgs) -> Result<()> {
    // Create output directory
    let output_dir = Path::new(&args.output_dir);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    // Read input file
    let input_file = File::open(&args.multifasta)?;
    let reader = BufReader::new(input_file);
    
    let mut current_id = String::new();
    let mut current_desc = String::new();
    let mut current_seq = String::new();
    let mut in_record = false;
    let mut count = 0;
    
    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with('>') {
            // Process previous record
            if in_record && !current_seq.is_empty() {
                write_sequence_file(
                    &current_id,
                    &current_desc,
                    &current_seq,
                    output_dir,
                    &args.extension,
                )?;
                count += 1;
            }
            
            // Parse new header
            let header = line[1..].trim();
            let parts: Vec<&str> = header.splitn(2, char::is_whitespace).collect();
            current_id = parts[0].to_string();
            current_desc = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
            current_seq.clear();
            in_record = true;
        } else if in_record {
            current_seq.push_str(&line);
        }
    }
    
    // Process last record
    if in_record && !current_seq.is_empty() {
        write_sequence_file(
            &current_id,
            &current_desc,
            &current_seq,
            output_dir,
            &args.extension,
        )?;
        count += 1;
    }
    
    println!("Successfully split {} into {} single FASTA files", 
             Path::new(&args.multifasta).file_name().unwrap_or_default().to_string_lossy(),
             count);
    
    Ok(())
}

fn write_sequence_file(
    id: &str,
    desc: &str,
    seq: &str,
    output_dir: &Path,
    extension: &str,
) -> Result<()> {
    // Sanitize filename: replace problematic characters
    let sanitized_id = id.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '.', "_");
    
    let filename = format!("{}.{}", sanitized_id, extension);
    let filepath = output_dir.join(filename);
    
    let file = File::create(&filepath)?;
    let mut writer = BufWriter::new(file);
    
    // Write full header with description if present
    if desc.is_empty() {
        writeln!(writer, ">{}", id)?;
    } else {
        writeln!(writer, ">{} {}", id, desc)?;
    }
    
    // Write sequence in chunks of 80 characters
    let chunk_size = 80;
    for chunk in seq.as_bytes().chunks(chunk_size) {
        writer.write_all(chunk)?;
        writeln!(writer)?;
    }
    
    Ok(())
}
