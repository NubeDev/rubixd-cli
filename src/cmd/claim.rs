//! `blockd-cli claim <token> --owner <name>` — `POST /api/v1/token/claim`.
//!
//! One-shot bootstrap: presents the `claim_token` from the device sticker and
//! receives the `owner_token` in the response. Store the returned token safely.
//! The API will respond 410 Gone on any subsequent calls.

use anyhow::Result;
use serde_json::json;

use crate::client::Client;
use crate::output::{self, Format};

/// Claim the device.
pub async fn run(client: &Client, token: &str, owner: &str, fmt: Format) -> Result<()> {
    let body = json!({ "claim_token": token, "owner": owner });
    let v = client.post_json("/api/v1/token/claim", &body).await?;

    fmt.print(&v, |v| {
        let recorded_owner = output::str_field(v, "owner");
        let owner_token = output::str_field(v, "owner_token");
        println!("claimed as      : {recorded_owner}");
        println!("owner_token     : {owner_token}");
        println!();
        println!("Store this token securely — it is returned exactly once.");
    });

    Ok(())
}
