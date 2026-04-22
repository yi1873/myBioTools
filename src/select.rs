use anyhow::{Result, anyhow};
use clap::Args;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::collections::HashSet;
use crate::utils;

#[derive(Args, Debug)]
#[command(
    about = "Extract target lines/sequences or filter by length from FASTA",
    long_about = "
Extract target lines/sequences or filter by length from FASTA

Modes:
  line    - Extract lines from a table file matching gene IDs in list
  fa      - Extract sequences from FASTA file matching gene IDs in list  
  onlyfa  - Filter FASTA sequences by minimum length (no gene ID list needed)

Examples:
  mybiotools select -cls line -l geneid.txt -s table.txt -o output.txt
  mybiotools select -cls fa -l geneid.txt -s sequences.fasta -o selected.fasta
  mybiotools select -cls onlyfa -s sequences.fasta -len 100 -o filtered.fasta
"
)]
pub struct SelectArgs {
    /// Operation mode: line, fa, or onlyfa
    #[arg(short = 'c', long = "cls", value_name = "MODE")]
    pub mode: String,
    
    /// Gene ID list file (required for line and fa modes)
    #[arg(short, long, value_name = "FILE")]
    pub list: Option<String>,
    
    /// Input object file (table for line mode, FASTA for fa/onlyfa modes)
    #[arg(short, long, value_name = "FILE")]
    pub source: String,
    
    /// Column number for gene ID in table (1-based, default: 1)
    #[arg(short, long, default_value_t = 1, value_name = "NUM")]
    pub column: usize,
    
    /// Minimum sequence length for onlyfa mode (default: 0)
    #[arg(long, default_value_t = 0, value_name = "LEN")]
    pub len: usize,
    
    /// Output file
    #[arg(short, long, value_name = "FILE")]
    pub output: String,
}

pub fn run(args: SelectArgs) -> Result<()> {
    match args.mode.as_str() {
        "line" => extract_lines(&args),
        "fa" => extract_fasta(&args),
        "onlyfa" => filter_fasta_by_length(&args),
        _ => Err(anyhow!("Invalid mode: {}. Must be 'line', 'fa', or 'onlyfa'", args.mode)),
    }
}

fn extract_lines(args: &SelectArgs) -> Result<()> {
    let gene_ids = match &args.list {
        Some(list_file) => utils::read_gene_ids(list_file)?,
        None => return Err(anyhow!("Gene ID list file (-l) is required for 'line' mode")),
    };
    
    let id_set: HashSet<String> = gene_ids.into_iter().collect();
    let source_file = File::open(&args.source)?;
    let reader = BufReader::new(source_file);
    let output_file = File::create(&args.output)?;
    let mut writer = BufWriter::new(output_file);
    
    for line in reader.lines() {
        let line = line?;
        
        // Skip comment lines starting with #
        if line.starts_with('#') {
            continue;
        }
        
        let columns: Vec<&str> = line.split('\t').collect();
        if args.column > 0 && args.column <= columns.len() {
            let gene_id = columns[args.column - 1].trim();
            if id_set.contains(gene_id) {
                writeln!(writer, "{}", line)?;
            }
        }
    }
    
    Ok(())
}

fn extract_fasta(args: &SelectArgs) -> Result<()> {
    let gene_ids = match &args.list {
        Some(list_file) => utils::read_gene_ids(list_file)?,
        None => return Err(anyhow!("Gene ID list file (-l) is required for 'fa' mode")),
    };
    
    let id_set: HashSet<String> = gene_ids.into_iter().collect();
    let source_file = File::open(&args.source)?;
    let reader = BufReader::new(source_file);
    let output_file = File::create(&args.output)?;
    let mut writer = BufWriter::new(output_file);
    
    let mut current_id = String::new();
    let mut current_seq = String::new();
    let mut in_record = false;
    
    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with('>') {
            // Process previous record
            if in_record && !current_seq.is_empty() {
                if id_set.contains(&current_id) {
                    utils::write_fasta(&mut writer, &current_id, None, &current_seq)?;
                }
            }
            
            // Parse new header
            let header = line[1..].trim();
            // Get first token as ID (split on whitespace)
            current_id = header.split_whitespace().next().unwrap_or(header).to_string();
            current_seq.clear();
            in_record = true;
        } else if in_record {
            current_seq.push_str(&line);
        }
    }
    
    // Process last record
    if in_record && !current_seq.is_empty() {
        if id_set.contains(&current_id) {
            utils::write_fasta(&mut writer, &current_id, None, &current_seq)?;
        }
    }
    
    Ok(())
}

fn filter_fasta_by_length(args: &SelectArgs) -> Result<()> {
    let min_len = args.len;
    let source_file = File::open(&args.source)?;
    let reader = BufReader::new(source_file);
    let output_file = File::create(&args.output)?;
    let mut writer = BufWriter::new(output_file);
    
    let mut current_id = String::new();
    let mut current_seq = String::new();
    let mut in_record = false;
    let mut seen_ids = HashSet::new();
    
    for line in reader.lines() {
        let line = line?;
        
        if line.starts_with('>') {
            // Process previous record
            if in_record && !current_seq.is_empty() {
                let seq_len = current_seq.len();
                if seq_len >= min_len && !seen_ids.contains(&current_id) {
                    // Replace U with T (RNA to cDNA compatibility)
                    let seq = current_seq.replace('U', "T").replace('u', "t");
                    utils::write_fasta(&mut writer, &current_id, None, &seq)?;
                    seen_ids.insert(current_id.clone());
                }
            }
            
            // Parse new header - get first token as ID
            let header = line[1..].trim();
            current_id = header.split_whitespace().next().unwrap_or(header).to_string();
            current_id = current_id.replace('/', "_"); // Replace / with _
            current_seq.clear();
            in_record = true;
        } else if in_record {
            // Remove digits and whitespace from sequence (as in original Perl)
            let cleaned_line: String = line.chars().filter(|c| !c.is_digit(10) && !c.is_whitespace()).collect();
            current_seq.push_str(&cleaned_line);
        }
    }
    
    // Process last record
    if in_record && !current_seq.is_empty() {
        let seq_len = current_seq.len();
        if seq_len >= min_len && !seen_ids.contains(&current_id) {
            let seq = current_seq.replace('U', "T").replace('u', "t");
            utils::write_fasta(&mut writer, &current_id, None, &seq)?;
        }
    }
    
    Ok(())
}
