use crate::state;
use dioxus::prelude::*;
use std::time::{SystemTime, SystemTimeError};
use zeroxname_ethereum::Address;

pub async fn resolve_name(input: String) -> Result<Address, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    let (name, community) = input.split_once("@").unwrap_or(("", ""));

    app_state
        .zx
        .resolve_address(name, community)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Unable to resolve the name: {}", e)))
}

pub async fn is_network_fees_ok() -> Result<bool, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    app_state
        .zx
        .is_network_fee_ok()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Unable to check Network Fees: {}", e)))
}

pub async fn send_sepolia_eth(addr: Address) -> Result<String, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    app_state
        .zx
        .send_sepolia_eth(addr)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Unable to send Sepolia ETH: {}", e)))
}

pub fn now_timestamp() -> Result<u64, SystemTimeError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(now)
}

pub async fn is_in_cooldown(last_record: u64) -> Result<bool, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    let cur_time = now_timestamp().map_err(|_| {
        ServerFnError::<std::io::Error>::ServerError("Failed to get current timestamp".to_string())
    })?;
    let is_cooldown = (cur_time - last_record) < app_state.cooldown_sec;
    Ok(is_cooldown)
}

pub async fn db_get_last_claim(key: &str) -> Result<u64, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    match app_state.db.get_value(key) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Ok(0),
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Get value from DB error: {}",
            e
        ))),
    }
}

pub async fn insert_timestamp(key: &str, value: u64) -> Result<(), ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    match app_state.db.insert_k_v(key, value) {
        Ok(_) => return Ok(()),
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Insert value to DB error: {}",
            e
        ))),
    }
}
