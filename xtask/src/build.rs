use std::process::Command;

use anyhow::Context as _;
use clap::Parser;

use crate::build_ebpf::{build_ebpf, Architecture, Options as BuildOptions};

#[derive(Debug, Parser)]
pub struct Options {
    /// Name of the BPF program
    pub name: String,
    /// Set the endianness of the BPF target
    #[clap(default_value = "bpfel-unknown-none", long)]
    pub bpf_target: Architecture,
    /// Build and run the release target
    #[clap(long)]
    pub release: bool,
    /// Produce binaries on release/tag
    #[clap(long)]
    pub produce_binaries: bool,
}

/// Build the project
fn build_project(opts: &Options) -> Result<(), anyhow::Error> {
    let mut args = vec!["build"];
    if opts.release {
        args.push("--release")
    }
    let status = Command::new("cargo")
        .current_dir(&opts.name)
        .args(&args)
        .status()
        .expect("failed to build userspace");
    assert!(status.success());
    Ok(())
}

/// Build our ebpf program and the project
pub fn build(opts: Options) -> Result<(), anyhow::Error> {
    // build our ebpf program followed by our application
    build_ebpf(BuildOptions {
        target: opts.bpf_target,
        release: opts.release,
        name: format!("{}-ebpf", opts.name),
        produce_binaries: opts.produce_binaries,
    })
    .context("Error while building eBPF program")?;
    build_project(&opts).context("Error while building userspace application")?;

    if opts.produce_binaries {
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .status()
            .expect("failed to produce binaries");
        assert!(status.success());
    }

    Ok(())
}
