//! `rubixd-cli audit [--limit N]` — `GET /api/v1/audit`.
//!
//! Tails the daemon audit log. Requires the owner token (G1).

use anyhow::Result;

use crate::client::Client;
use crate::output::Format;

/// Fetch and display audit log entries.
pub async fn run(client: &Client, limit: usize, fmt: Format) -> Result<()> {
    let v = client
        .get_query("/api/v1/audit", &[("limit", limit.to_string())])
        .await?;

    fmt.print(&v, |v| {
        let entries = v.get("entries").and_then(|e| e.as_array());
        match entries {
            None => println!("(no entries)"),
            Some(list) if list.is_empty() => println!("(no entries)"),
            Some(list) => {
                for entry in list {
                    // Entries are opaque strings from the daemon; print as-is.
                    if let Some(s) = entry.as_str() {
                        println!("{s}");
                    } else {
                        println!("{entry}");
                    }
                }
            }
        }
    });

    Ok(())
}
