//! `n2w` / `nixpkgs2wasi`: curated nixpkgs → WASI packages for Wawona.
//!
//! North star: `n2w build nixpkgs#foot` → `foot.wpm` → `wpm install foot`
//! → a Wayland window on Wawona Compositor. No Linux VM.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use n2w_analyze::{analyze, catalog_dir, list_packages, load_package};
use n2w_build::{build as run_build, plan_build, BuildError};
use n2w_verify::verify_appstore_runtime;

#[derive(Parser)]
#[command(
    name = "n2w",
    about = "Convert curated nixpkgs Wayland clients to WASI for Wawona Relay",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List curated packages (not an automatic nixpkgs mirror).
    Catalog,
    /// Show one package.toml.
    Show { package: String },
    /// Classify WASI / POSIX / Wayland / graphics tier.
    Analyze { package: String },
    /// Print the cross / WPM plan without writing artifacts.
    Plan { package: String },
    /// Verify the Wawona App Store runtime profile (not Apple review).
    Verify { package: String },
    /// Cross-build. Fail closed until the WASI closure actually compiles.
    Build { package: String },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{err:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.command {
        Command::Catalog => {
            let catalog = catalog_dir()?;
            for name in list_packages(&catalog)? {
                let spec = load_package(&catalog, &name)?;
                println!(
                    "{:<18} {:<10} {}",
                    spec.package.name,
                    format!("{:?}", spec.package.status).to_lowercase(),
                    spec.package.source
                );
            }
        }
        Command::Show { package } => {
            let catalog = catalog_dir()?;
            let spec = load_package(&catalog, &package)?;
            println!("{}", toml::to_string_pretty(&spec)?);
        }
        Command::Analyze { package } => {
            let analysis = analyze(&package)?;
            println!("package:     {}", analysis.spec.package.name);
            println!("source:      {}", analysis.spec.package.source);
            println!("status:      {:?}", analysis.spec.package.status);
            println!("tier:        {}", analysis.tier.as_str());
            println!("llvm:        {}", analysis.llvm_target);
            println!("platform:    wasm32-wawona (conceptual)");
            println!("native port: {}", analysis.spec.package.native_port);
            println!("wayland:     {}", analysis.spec.wawona.wayland);
        }
        Command::Plan { package } => {
            print_plan(&package)?;
        }
        Command::Verify { package } => {
            let analysis = analyze(&package)?;
            let report = verify_appstore_runtime(&analysis)?;
            println!("{}: ok", report.profile);
            for note in report.notes {
                println!("- {note}");
            }
        }
        Command::Build { package } => {
            print_plan(&package)?;
            match run_build(&package) {
                Ok(_) => unreachable!("cross-compile is not implemented"),
                Err(BuildError::AlreadyShipping(name)) => {
                    println!();
                    println!("{name} is already on repo.wawona.io/wasm/v1 via Relay.");
                    println!("Install with: wpm install {name}");
                    return Ok(ExitCode::SUCCESS);
                }
                Err(BuildError::CrossNotReady { name }) => {
                    eprintln!();
                    eprintln!("Refusing to write a stub {name}.wasm.");
                    eprintln!("WASI cross of the nixpkgs userspace closure is the remaining work.");
                    return Ok(ExitCode::from(2));
                }
                Err(err) => return Err(err.into()),
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_plan(package: &str) -> Result<()> {
    let plan = plan_build(package)?;
    println!("# {} ({})", plan.analysis.spec.package.name, plan.analysis.tier.as_str());
    println!("source: {}", plan.analysis.spec.package.source);
    println!("llvm:   {}", plan.analysis.llvm_target);
    if let Some(nix) = &plan.nix {
        println!("nix:    {} on {}", nix.attr, nix.conceptual_platform);
        println!("note:   {}", nix.note);
    }
    for note in &plan.patch.notes {
        println!("patch:  {note}");
    }
    println!("verify: {} ok", plan.verify.profile);
    println!();
    println!("{}", plan.artifact_tree);
    println!("{}", plan.manifest.to_toml()?);
    Ok(())
}
