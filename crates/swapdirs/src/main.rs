use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use swapcore::{SwapConfig, feasibility_check, run_dry, run_real};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
  #[arg(long)] a_root: PathBuf,
  #[arg(long)] b_root: PathBuf,
  #[arg(long)] a_dest: PathBuf,
  #[arg(long)] b_dest: PathBuf,

  #[arg(long, default_value_t = 1024)]
  reserve_mib: u64,

  #[arg(long, default_value_t = 8)]
  buffer_mib: usize,

  #[arg(long)]
  dry_run: bool,

  #[arg(long)]
  verbose: bool,
}

fn main() -> Result<()> {
  let args = Args::parse();

  let cfg = SwapConfig {
    a_root: args.a_root,
    b_root: args.b_root,
    a_dest: args.a_dest,
    b_dest: args.b_dest,
    reserve_bytes: args.reserve_mib * 1024 * 1024,
    buffer_bytes: args.buffer_mib * 1024 * 1024,
    verbose: args.verbose,
  };

  let info = feasibility_check(&cfg)?;
  println!("{}", info.summary);

  if !info.ok {
    return Ok(());
  }

  if args.dry_run {
    let report = run_dry(&cfg)?;
    println!("{}", report.summary);
  } else {
    let report = run_real(&cfg)?;
    println!("{}", report.summary);
  }

  Ok(())
}
