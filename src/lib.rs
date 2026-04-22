//! myBioTools - A collection of bioinformatics tools implemented in Rust

pub mod select;
pub mod split_fasta;
pub mod tm;
pub mod fasta_stats;
pub mod hairpin;
pub mod heterodimer;
pub mod homodimer;
pub mod rev_comp;

// Common utilities
pub mod utils {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Write};
    use anyhow::Result;

    /// Read gene IDs from a file (one per line)
    pub fn read_gene_ids(path: &str) -> Result<Vec<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut ids = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                ids.push(trimmed.to_string());
            }
        }
        
        Ok(ids)
    }

    /// Write FASTA record
    pub fn write_fasta<W: Write>(writer: &mut W, id: &str, description: Option<&str>, seq: &str) -> Result<()> {
        if let Some(desc) = description {
            writeln!(writer, ">{} {}", id, desc)?;
        } else {
            writeln!(writer, ">{}", id)?;
        }
        
        // Write sequence in chunks of 80 characters
        let chunk_size = 80;
        for chunk in seq.as_bytes().chunks(chunk_size) {
            writer.write_all(chunk)?;
            writeln!(writer)?;
        }
        
        Ok(())
    }
}
