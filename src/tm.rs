use anyhow::{Result, anyhow};
use clap::Args;
use pyo3::prelude::*;

#[derive(Args, Debug)]
#[command(
    about = "Calculate melting temperature (Tm) of DNA sequences",
    long_about = "
Calculate melting temperature (Tm) of DNA sequences

Uses primer3 bindings with default parameters:
  Na+ concentration: 50 mM
  Primer concentration: 50 nM
  Mg2+ concentration: 1.5 mM
  dNTPs concentration: 0.0 mM
"
)]
pub struct TmArgs {
    /// Input DNA sequence
    #[arg(short, long, value_name = "SEQ")]
    pub input: String,
    
    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: TmArgs) -> Result<()> {
    let tm = calculate_tm(&args.input)?;
    
    if args.json {
        let json = serde_json::json!({
            "sequence": args.input,
            "tm_celsius": format!("{:.2}", tm),
            "unit": "°C"
        });
        println!("{}", json.to_string());
    } else {
        println!("Sequence: {}", args.input);
        println!("Tm: {:.2} °C", tm);
    }
    
    Ok(())
}

fn calculate_tm(seq: &str) -> Result<f64> {
    Python::with_gil(|py| -> PyResult<f64> {
        let primer3 = PyModule::import_bound(py, "primer3.bindings")?;
        let tm = primer3.getattr("calcTm")?.call1((seq,))?;
        tm.extract()
    }).map_err(|e| anyhow!("Failed to calculate Tm: {}", e))
}
