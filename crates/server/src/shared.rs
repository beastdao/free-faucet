#[cfg(feature = "server")]
use crate::server::*;
#[cfg(feature = "server")]
use crate::state;
use dioxus::prelude::*;

#[server(Claim)]
pub async fn claim_server(input: String) -> Result<String, ServerFnError> {
    let time = now_timestamp()?;
    let fees_ok = is_network_fees_ok().await?;
    let result = async {
        let addr = resolve_name(input.clone()).await?;
        let string_address = addr.to_string();

        if is_in_cooldown(db_get_last_claim(&string_address).await?).await? {
            return Err(ServerFnError::ServerError("Cooldown is not ended!".into()));
        }

        if !fees_ok {
            return Err(ServerFnError::ServerError(
                "Network Fee is too high!".into(),
            ));
        }
        let hash = send_sepolia_eth(addr).await?;
        insert_timestamp(&string_address, time).await?;

        Ok(hash)
    }
    .await;
    match &result {
        Ok(hash) => {
            insert_log(time, true, input, hash.to_string()).await.ok();
        }
        Err(e) => {
            insert_log(time, false, input, e.to_string()).await.ok();
        }
    }
    result
}

use shared_types::LogEntry;

#[server(GetLogs)]
pub async fn get_all_logs() -> Result<Vec<LogEntry>, ServerFnError> {
    Ok(get_logs().await?)
}
