#[cfg(feature = "server")]
use crate::server::*;
#[cfg(feature = "server")]
use crate::state;
use dioxus::prelude::*;

#[server(Claim)]
pub async fn claim_server(input: String) -> Result<String, ServerFnError> {
    let addr = resolve_name(input).await?;
    let string_address = addr.to_string();
    let last_claim = db_get_last_claim(&string_address).await?;
    if is_in_cooldown(last_claim).await? {
        return Err(ServerFnError::ServerError(
            "Cooldown is not ended!".to_string(),
        ));
    }
    let fees_ok = is_network_fees_ok().await?;
    if !fees_ok {
        return Err(ServerFnError::ServerError(
            "Network Fee is too high!".to_string(),
        ));
    }
    let hash = send_sepolia_eth(addr).await?;
    insert_timestamp(&string_address, now_timestamp()?).await?;
    Ok(hash)
}
