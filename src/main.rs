//! `rubixd-cli` — remote management CLI for the rubixd supervisor daemon.
//!
//! Entry point: parse the command tree with clap, build a [`Client`], dispatch
//! to the correct `cmd::*` function, and print the result with [`output`].
//! No business logic lives here.

mod client;
mod cmd;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Remote management CLI for the rubixd supervisor daemon.
#[derive(Debug, Parser)]
#[command(name = "rubixd-cli", version, about)]
pub struct Cli {
    /// rubixd API base URL.
    #[arg(long, env = "BLOCKD_URL", global = true, default_value = "http://127.0.0.1:9999")]
    pub url: String,

    /// Owner bearer token for authenticated endpoints.
    #[arg(long, env = "BLOCKD_TOKEN", global = true)]
    pub token: Option<String>,

    /// Emit raw JSON instead of human-readable output.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print daemon status.
    Status,
    /// Claim an unclaimed device and receive the owner token.
    Claim {
        /// The one-shot claim token (printed at first boot).
        token: String,
        /// Owner name to record.
        #[arg(long)]
        owner: String,
    },
    /// Bundle management.
    Bundle {
        #[command(subcommand)]
        sub: cmd::bundle::BundleCmd,
    },
    /// Update consent actions.
    Update {
        #[command(subcommand)]
        sub: cmd::update::UpdateCmd,
    },
    /// Tail the audit log.
    Audit {
        /// Maximum number of entries to return (1–1000).
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// TLS certificate operations.
    Tls {
        #[command(subcommand)]
        sub: cmd::tls::TlsCmd,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = client::Client::new(&cli.url, cli.token.as_deref())?;
    let fmt = output::Format::from_json_flag(cli.json);

    match cli.command {
        Command::Status => cmd::status::run(&client, fmt).await,
        Command::Claim { token, owner } => cmd::claim::run(&client, &token, &owner, fmt).await,
        Command::Bundle { sub } => cmd::bundle::run(&client, sub, fmt).await,
        Command::Update { sub } => cmd::update::run(&client, sub, fmt).await,
        Command::Audit { limit } => cmd::audit::run(&client, limit, fmt).await,
        Command::Tls { sub } => cmd::tls::run(&client, sub, fmt).await,
    }
}
