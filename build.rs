use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};
use clap_mangen::Man;
use std::fs::File;
use std::io::Error;
use std::path::Path;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    // `shared_extents`: --shared-blocks counts physical extents in a byte-range
    // map instead of APFS clone ids. Linux has no clone ids, so it always does.
    println!("cargo::rustc-check-cfg=cfg(shared_extents)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux")
        || std::env::var_os("CARGO_FEATURE_EXACT_SHARED_BLOCKS").is_some()
    {
        println!("cargo::rustc-cfg=shared_extents");
    }

    let outdir = "completions";
    let app_name = "dust";
    let mut cmd = Cli::command();

    generate_to(Bash, &mut cmd, app_name, outdir)?;
    generate_to(Zsh, &mut cmd, app_name, outdir)?;
    generate_to(Fish, &mut cmd, app_name, outdir)?;
    generate_to(PowerShell, &mut cmd, app_name, outdir)?;
    generate_to(Elvish, &mut cmd, app_name, outdir)?;

    let file = Path::new("man-page").join("dust.1");
    std::fs::create_dir_all("man-page")?;
    let mut file = File::create(file)?;

    Man::new(cmd).render(&mut file)?;

    Ok(())
}
