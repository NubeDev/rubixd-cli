//! `blockd-cli update accept|skip|rollback` — `/api/v1/update/*`.
//!
//! All three verbs require the owner token (G1).

use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

use crate::client::Client;
use crate::output::{self, Format};

/// Update consent subcommands.
#[derive(Debug, Subcommand)]
pub enum UpdateCmd {
    /// Accept the staged bundle: flip the active slot.
    Accept {
        /// Run the pre-apply backup hook before flipping.
        #[arg(long)]
        backup: bool,
    },
    /// Skip the staged bundle without applying it.
    Skip,
    /// Roll back to the inactive slot (previous version).
    Rollback,
}

/// Dispatch update subcommands.
pub async fn run(client: &Client, cmd: UpdateCmd, fmt: Format) -> Result<()> {
    match cmd {
        UpdateCmd::Accept { backup } => accept(client, backup, fmt).await,
        UpdateCmd::Skip => skip(client, fmt).await,
        UpdateCmd::Rollback => rollback(client, fmt).await,
    }
}

async fn accept(client: &Client, backup: bool, fmt: Format) -> Result<()> {
    let body = json!({ "backup": backup });
    let v = client.post_json("/api/v1/update/accept", &body).await?;

    fmt.print(&v, |v| {
        let slot = output::str_field(v, "active_slot");
        let ver = output::str_field(v, "applied_version");
        println!("applied_version : {ver}");
        println!("active_slot     : {slot}");
    });

    Ok(())
}

async fn skip(client: &Client, fmt: Format) -> Result<()> {
    let v = client.post_empty("/api/v1/update/skip").await?;

    fmt.print(&v, |v| {
        let skipped = output::bool_field(v, "skipped");
        if skipped {
            println!("staged bundle cleared.");
        } else {
            println!("nothing staged to skip.");
        }
    });

    Ok(())
}

async fn rollback(client: &Client, fmt: Format) -> Result<()> {
    let v = client.post_empty("/api/v1/update/rollback").await?;

    fmt.print(&v, |v| {
        let slot = output::str_field(v, "active_slot");
        let rolled_back = output::bool_field(v, "rolled_back");
        println!("rolled_back : {rolled_back}");
        println!("active_slot : {slot}");
    });

    Ok(())
}
