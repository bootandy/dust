use clap_complete::{generate_to, shells::*};
use std::io::Error;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = "completions";
    let app_name = "dust";
    let max_depth = usize::MAX.to_string();
    let mut cmd = build_cli(&max_depth);

    generate_to(Bash, &mut cmd, app_name, outdir)?;
    generate_to(Zsh, &mut cmd, app_name, outdir)?;
    generate_to(Fish, &mut cmd, app_name, outdir)?;
    generate_to(PowerShell, &mut cmd, app_name, outdir)?;
    generate_to(Elvish, &mut cmd, app_name, outdir)?;

    Ok(())
}
