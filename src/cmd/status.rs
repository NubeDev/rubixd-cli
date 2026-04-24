//! `blockd-cli status` — `GET /api/v1/status`.
//!
//! No auth required (G0) when connecting to the loopback listener.

use anyhow::Result;

use crate::client::Client;
use crate::output::{self, Format};

/// Fetch and display daemon status.
pub async fn run(client: &Client, fmt: Format) -> Result<()> {
    let v = client.get("/api/v1/status").await?;

    fmt.print(&v, |v| {
        let claimed = output::bool_field(v, "claimed");
        let can_claim = output::bool_field(v, "can_claim");
        let active_slot = output::str_field(v, "active_slot");
        let version = output::str_field(v, "version");

        println!("blockd version  : {version}");
        println!("claimed         : {claimed}");
        println!("can_claim       : {can_claim}");
        println!("active_slot     : {active_slot}");

        if let Some(ts) = v.get("last_apply_unix").and_then(|x| x.as_u64()) {
            println!("last_apply_unix : {ts}");
        } else {
            println!("last_apply_unix : -");
        }

        if let Some(staged) = v.get("staged").filter(|s| !s.is_null()) {
            let sv = output::str_field(staged, "version");
            let ss = output::str_field(staged, "subject");
            let ssev = output::str_field(staged, "severity");
            println!("staged          : {ss} {sv} (severity: {ssev})");
        } else {
            println!("staged          : (none)");
        }
    });

    Ok(())
}
