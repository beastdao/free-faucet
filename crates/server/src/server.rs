use crate::state;
use dioxus::prelude::*;
use shared_types::{LogEntry, LogValue, PayoutRange};
use std::time::{SystemTime, SystemTimeError};
use zeroxname_ethereum::Address;
use zeroxname_ethereum::U256;
use zeroxname_ethereum::format_units;

//replace to config vars
const PERIOD_SEC: u64 = 86400; // 86400 secs = 24 hours
const STEP_SEC: u64 = 3600; //3600 secs = 1 hour
const STEPS_AMOUNT: u64 = PERIOD_SEC / STEP_SEC; //24

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
    let coefficient = calculate_current_coefficient().await?;
    app_state
        .zx
        .send_sepolia_eth(addr, coefficient)
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
    match app_state.db.get_value_claim(key) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Ok(0),
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Get claim value from DB error: {}",
            e
        ))),
    }
}

pub async fn insert_timestamp(key: &str, value: u64) -> Result<(), ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    match app_state.db.insert_k_v_claim(key, value) {
        Ok(_) => return Ok(()),
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Insert claim value to DB error: {}",
            e
        ))),
    }
}

async fn calculate_current_coefficient() -> Result<f64, ServerFnError> {
    let now = now_timestamp().map_err(|_| {
        ServerFnError::<std::io::Error>::ServerError("Failed to get current timestamp".to_string())
    })?;

    let range_low = now - PERIOD_SEC;
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;

    match app_state.db.get_last_claim_timestamp(range_low, now) {
        Ok(last_claim_ts) => {
            let secs_elapsed = now.saturating_sub(last_claim_ts);
            let steps_elapsed = (secs_elapsed / STEP_SEC).clamp(0, STEPS_AMOUNT);
            let coefficient = steps_elapsed as f64 * app_state.payout_adjustment;
            Ok(coefficient)
        }
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Getting time elapsed from last claim: {}",
            e
        ))),
    }
}

pub async fn get_payout_range() -> Result<PayoutRange, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    let current_coefficient = calculate_current_coefficient().await?;
    let min_coef = 0.0;
    let cur_coef = current_coefficient;
    let max_coef = STEPS_AMOUNT as f64 * app_state.payout_adjustment;

    let format_amount = |coef: f64| -> Result<String, ServerFnError> {
        let wei = app_state.zx.get_claim_amount(coef);
        format_units(wei, "ether")
            .map_err(|e| ServerFnError::ServerError(format!("Failed to format units: {}", e)))
    };

    Ok(PayoutRange {
        min: format_amount(min_coef)?,
        current: format_amount(cur_coef)?,
        max: format_amount(max_coef)?,
    })
}

pub async fn insert_log(
    timestamp: u64,
    status: bool,
    input: String,
    result: String,
) -> Result<(), ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;

    match app_state
        .db
        .insert_k_v_logs(timestamp, status, input, result)
    {
        Ok(_) => return Ok(()),
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Insert log to DB error: {}",
            e
        ))),
    }
}

pub async fn get_logs() -> Result<Vec<LogEntry>, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    let logs: Vec<LogEntry> = app_state.db.iter_logs().collect::<Result<Vec<_>, _>>()?;
    Ok(logs)
}
