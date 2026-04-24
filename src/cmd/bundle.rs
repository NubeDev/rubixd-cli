//! `blockd-cli bundle push <file>` — `POST /api/v1/bundles/push`.
//!
//! Reads the bundle file from disk, streams it to the daemon with a progress
//! bar, and prints the staged bundle metadata on success.

use anyhow::{Context, Result};
use clap::Subcommand;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::client::Client;
use crate::output::{self, Format};

/// Bundle subcommands.
#[derive(Debug, Subcommand)]
pub enum BundleCmd {
    /// Push a `.block-os` bundle to the daemon for staging.
    Push {
        /// Path to the `.block-os` bundle file.
        file: PathBuf,
    },
}

/// Dispatch bundle subcommands.
pub async fn run(client: &Client, cmd: BundleCmd, fmt: Format) -> Result<()> {
    match cmd {
        BundleCmd::Push { file } => push(client, &file, fmt).await,
    }
}

async fn push(client: &Client, file: &PathBuf, fmt: Format) -> Result<()> {
    let bytes = tokio::fs::read(file)
        .await
        .with_context(|| format!("reading bundle {}", file.display()))?;

    let total = bytes.len() as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("valid template")
            .progress_chars("#>-"),
    );
    pb.set_position(0);

    // Tick so the bar renders immediately even for small files.
    pb.tick();

    let v = client.post_bytes("/api/v1/bundles/push", bytes).await?;
    pb.finish_and_clear();

    fmt.print(&v, |v| {
        let staged = output::bool_field(v, "staged");
        let version = output::str_field(v, "version");
        let subject = output::str_field(v, "subject");
        let severity = output::str_field(v, "severity");
        println!("staged   : {staged}");
        println!("subject  : {subject}");
        println!("version  : {version}");
        println!("severity : {severity}");
    });

    Ok(())
}
