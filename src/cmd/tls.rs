//! `rubixd-cli tls fingerprint` — `GET /api/v1/tls/fingerprint`.
//!
//! Reports the LAN-mode TLS certificate fingerprint for out-of-band pinning.
//! Requires the owner token (G1).

use anyhow::Result;
use clap::Subcommand;

use crate::client::Client;
use crate::output::{self, Format};

/// TLS subcommands.
#[derive(Debug, Subcommand)]
pub enum TlsCmd {
    /// Print the current LAN-mode certificate fingerprint.
    Fingerprint,
}

/// Dispatch TLS subcommands.
pub async fn run(client: &Client, cmd: TlsCmd, fmt: Format) -> Result<()> {
    match cmd {
        TlsCmd::Fingerprint => fingerprint(client, fmt).await,
    }
}

async fn fingerprint(client: &Client, fmt: Format) -> Result<()> {
    let v = client.get("/api/v1/tls/fingerprint").await?;

    fmt.print(&v, |v| {
        let fp = output::str_field(v, "fingerprint");
        let path = output::str_field(v, "cert_path");
        println!("fingerprint : {fp}");
        println!("cert_path   : {path}");
    });

    Ok(())
}
