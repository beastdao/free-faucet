use dioxus::prelude::*;

#[cfg(feature = "server")]
use std::time::{ SystemTime, SystemTimeError};
#[cfg(feature = "server")]
use zeroxname_ethereum::Address;
#[cfg(feature = "server")]
pub mod state {
    use db::*;
    use dotenv::dotenv;
    use std::env;
    use zeroxname_ethereum::*;

    #[derive(Debug, thiserror::Error)]
    pub enum AppStateErrors {
        #[error("DB init Error: {0}")]
        DBError(#[from] DBErrors),
        #[error("Eth lib Init Error: {0}")]
        ETHError(#[from] EthErrors),
        #[error("Env vars Init Error: {0}")]
        ENVError(#[from] std::env::VarError),
        #[error("Parse error: should be u64")]
        ENVIntError(#[from] std::num::ParseIntError),
        #[error("Parse error: should be f64")]
        ENVFloatError(#[from] std::num::ParseFloatError),
    }

    #[derive(Clone)]
    pub struct AppState {
        pub zx: ZeroxnameEthereum,
        pub db: DB,
        pub cooldown_sec: u64,
    }

    impl AppState {
        pub fn new() -> Result<Self, AppStateErrors> {
            dotenv().ok();
            let rpc_key = env::var("RPC_KEY")?;
            let private_key = env::var("PRIVATE_KEY")?;
            let db_path = env::var("DB_PATH")?;
            let faucet_limit: u64 = env::var("FAUCET_LIMIT")?.parse()?;
            let fee_threshold: f64 = env::var("FEE_THRESHOLD")?.parse()?;
            let cooldown_sec: u64 = env::var("COOLDOWN_SEC")?.parse()?;
            let z = ZeroxnameEthereum::new(&rpc_key, &private_key, faucet_limit, fee_threshold)?;
            let d = DB::new(&db_path, "registry")?;

            Ok(Self {
                zx: z,
                db: d,
                cooldown_sec: cooldown_sec,
            })
        }
    }
}

#[cfg(feature = "server")]
async fn resolve_name(input: String) -> Result<Address, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    let (name, community) = input.split_once("@").unwrap_or(("", ""));

    app_state
        .zx
        .resolve_address(name, community)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Unable to resolve the name: {}", e)))
}

#[cfg(feature = "server")]
async fn is_network_fees_ok() -> Result<bool, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;

    app_state
        .zx
        .is_network_fee_ok()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Unable to check Network Fees: {}", e)))
}

#[cfg(feature = "server")]
async fn send_sepolia_eth(addr: Address) -> Result<String, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    app_state
        .zx
        .send_sepolia_eth(addr)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Unable to send Sepolia ETH: {}", e)))
}

#[cfg(feature = "server")]
fn now_timestamp() -> Result<u64, SystemTimeError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(now)
}

#[cfg(feature = "server")]
async fn is_in_cooldown(last_record: u64) -> Result<bool, ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    let cur_time = now_timestamp().map_err(|_| {
        ServerFnError::<std::io::Error>::ServerError("Failed to get current timestamp".to_string())
    })?;
    let is_cooldown = (cur_time - last_record) < app_state.cooldown_sec;
    Ok(is_cooldown)
}

#[cfg(feature = "server")]
async fn db_get_last_claim(key: &str) -> Result<u64, ServerFnError> {
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

#[cfg(feature = "server")]
async fn insert_timestamp(key: &str, value: u64) -> Result<(), ServerFnError> {
    let FromContext(app_state): dioxus::prelude::FromContext<state::AppState> = extract().await?;
    match app_state.db.insert_k_v(key, value) {
        Ok(_) => return Ok(()),
        Err(e) => Err(ServerFnError::ServerError(format!(
            "Insert value to DB error: {}",
            e
        ))),
    }
}

#[server(Echo)]
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
