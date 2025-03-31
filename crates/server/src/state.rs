use db::*;
use dotenv::dotenv;
use std::env;
use zeroxname_ethereum::Address;
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
        let zx = ZeroxnameEthereum::new(&rpc_key, &private_key, faucet_limit, fee_threshold)?;
        let db = DB::new(&db_path, "registry")?;

        Ok(Self {
            zx,
            db,
            cooldown_sec,
        })
    }
}
