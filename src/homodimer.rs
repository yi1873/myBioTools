use anyhow::{Result, anyhow};
use clap::Args;
use pyo3::prelude::*;

#[derive(Args, Debug)]
#[command(
    about = "Calculate homodimerization ΔG (kcal/mol)",
    long_about = "
Calculate the homodimerization thermodynamics (ΔG in kcal/mol) of primer.
"
)]
pub struct HomodimerArgs {
    /// Input primer sequence
    #[arg(short, long, value_name = "SEQ")]
    pub input: String,
    
    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: HomodimerArgs) -> Result<()> {
    let dg_kcal = calculate_homodimer(&args.input)?;
    
    if args.json {
        let json = serde_json::json!({
            "sequence": args.input,
            "delta_g": format!("{:.2}", dg_kcal),
            "unit": "kcal/mol",
            "process": "homodimerization"
        });
        println!("{}", json.to_string());
    } else {
        println!("Sequence: {}", args.input);
        println!("Homodimerization ΔG: {:.2} kcal/mol", dg_kcal);
    }
    
    Ok(())
}

fn calculate_homodimer(seq: &str) -> Result<f64> {
    Python::with_gil(|py| -> PyResult<f64> {
        let primer3 = PyModule::import_bound(py, "primer3.bindings")?;
        let homodimer = primer3.getattr("calc_homodimer")?.call1((seq,))?;
        let dg: f64 = homodimer.getattr("dg")?.extract()?;
        Ok(dg / 1000.0) // Convert from cal/mol to kcal/mol
    }).map_err(|e| anyhow!("Failed to calculate homodimer ΔG: {}", e))
}
