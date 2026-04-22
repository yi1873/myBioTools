use anyhow::{Result, anyhow};
use clap::Args;
use pyo3::prelude::*;

#[derive(Args, Debug)]
#[command(
    about = "Calculate heterodimerization ΔG (kcal/mol)",
    long_about = "
Calculate the heterodimerization thermodynamics (ΔG in kcal/mol) of primer pairs.
"
)]
pub struct HeterodimerArgs {
    /// Forward primer sequence
    #[arg(short = 'f', long = "forward", value_name = "SEQ")]
    pub forward: String,
    
    /// Reverse primer sequence
    #[arg(short = 'r', long = "reverse", value_name = "SEQ")]
    pub reverse: String,
    
    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: HeterodimerArgs) -> Result<()> {
    let dg_kcal = calculate_heterodimer(&args.forward, &args.reverse)?;
    
    if args.json {
        let json = serde_json::json!({
            "forward": args.forward,
            "reverse": args.reverse,
            "delta_g": format!("{:.2}", dg_kcal),
            "unit": "kcal/mol",
            "process": "heterodimerization"
        });
        println!("{}", json.to_string());
    } else {
        println!("Forward: {}", args.forward);
        println!("Reverse: {}", args.reverse);
        println!("Heterodimerization ΔG: {:.2} kcal/mol", dg_kcal);
    }
    
    Ok(())
}

fn calculate_heterodimer(forward: &str, reverse: &str) -> Result<f64> {
    Python::with_gil(|py| -> PyResult<f64> {
        let primer3 = PyModule::import_bound(py, "primer3.bindings")?;
        let heterodimer = primer3.getattr("calc_heterodimer")?.call1((forward, reverse))?;
        let dg: f64 = heterodimer.getattr("dg")?.extract()?;
        Ok(dg / 1000.0) // Convert from cal/mol to kcal/mol
    }).map_err(|e| anyhow!("Failed to calculate heterodimer ΔG: {}", e))
}
