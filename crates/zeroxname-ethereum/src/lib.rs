use NamesRegistry::NamesRegistryInstance;
use alloy_dyn_abi::DynSolValue;
use alloy_network::{EthereumWallet, TransactionBuilder};
pub use alloy_primitives::Address;
pub use alloy_primitives::ruint::aliases::U256;
pub use alloy_primitives::utils::format_units;
use alloy_primitives::{address, keccak256};
use alloy_provider::{
    Identity, PendingTransactionError, Provider, ProviderBuilder, RootProvider,
    fillers::{
        BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
    },
};
use alloy_rpc_types::TransactionRequest;
use alloy_signer_local::{LocalSignerError, PrivateKeySigner};
use alloy_sol_macro::*;
use alloy_sol_types::*;
use alloy_transport::{RpcError, TransportErrorKind};

use std::{str::FromStr, string::ParseError};
use thiserror::Error;

pub static TX_GAS: u128 = 21000;
pub static NAMES_REGISTRY_CONTRACT_ADDRESS: Address =
    address!("0x636518cb98F2F705082da540ba961E0A608C8220");

#[derive(Debug, Error)]
pub enum EthErrors {
    #[error("Contract call failed: {0}")]
    ContractCallError(#[from] alloy_contract::Error),
    #[error("No name found")]
    NameNotFound,
    #[error("RPC ERROR: {0}")]
    RpcError(#[from] RpcError<TransportErrorKind>),
    #[error("Pending transaction ERROR: {0}")]
    PendingTxError(#[from] PendingTransactionError),
    #[error("Error getting token id hash")]
    TokenIdError,
    #[error("Error initing Signer: {0}")]
    InitSignerError(#[from] LocalSignerError),
    #[error("Error initing Provider: {0}")]
    InitParseURLError(#[from] ParseError),
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    NamesRegistry,
    "assets/abis/NamesRegistry.json"
);

type DefaultFiller = JoinFill<
    Identity,
    JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
>;

type DefaultFillerSender = JoinFill<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    WalletFiller<EthereumWallet>,
>;

#[derive(Clone, Debug)]
pub struct ZeroxnameEthereum {
    names_registry: NamesRegistryInstance<(), FillProvider<DefaultFiller, RootProvider>>,
    sepolia_sender: FillProvider<DefaultFillerSender, RootProvider>,
    faucet_limit: u64,
    fee_threshold: f64,
}

fn encode_string_to_bytes32(name: &str, community: &str) -> Vec<u8> {
    let name_at_community = DynSolValue::Tuple(vec![
        DynSolValue::String(name.to_string()),
        DynSolValue::String(community.to_string()),
    ]);
    name_at_community.abi_encode_packed()
}

fn hash(name: &str, community: &str) -> Result<U256, EthErrors> {
    let encoded = encode_string_to_bytes32(name, community);
    let hash = keccak256(encoded);
    let f: [u8; 32] = hash.try_into().map_err(|_| EthErrors::TokenIdError)?;
    Ok(U256::from_be_bytes(f))
}

//add error handling Result <Self,EthErrors>?
impl ZeroxnameEthereum {
    pub fn new(
        rpc_mainnet: &str,
        rpc_sepolia: &str,
        private_key: &str,
        faucet_amount: u64,
        fee_max: f64,
    ) -> Result<Self, EthErrors> {
        let signer =
            PrivateKeySigner::from_str(private_key).map_err(|e| EthErrors::InitSignerError(e))?;

        let wallet = EthereumWallet::from(signer);

        let sepolia_provider = ProviderBuilder::new()
            .wallet(wallet)
            .on_http(rpc_sepolia.parse().expect("Failed to parse sepolia URL"));

        let provider = ProviderBuilder::new()
            .on_http(rpc_mainnet.parse().expect("Failed to parse mainnet URL"));

        let names_registry_instance =
            NamesRegistry::new(NAMES_REGISTRY_CONTRACT_ADDRESS, provider.clone());
        Ok(Self {
            names_registry: names_registry_instance,
            sepolia_sender: sepolia_provider,
            faucet_limit: faucet_amount,
            fee_threshold: fee_max,
        })
    }

    pub async fn resolve_address(&self, name: &str, community: &str) -> Result<Address, EthErrors> {
        let token_id = hash(name, community)?;
        let user_addr = self
            .names_registry
            .resolveAddressByTokenId(token_id)
            .call()
            .await
            .map_err(|e| EthErrors::ContractCallError(e))?
            ._0;

        match user_addr.is_zero() {
            true => Err(EthErrors::NameNotFound),
            _ => Ok(user_addr),
        }
    }

    pub fn get_claim_amount(&self, coefficient: f64) -> U256 {
        let base_value = self.faucet_limit as f64;
        U256::from(base_value + base_value * coefficient)
    }

    pub async fn send_sepolia_eth(
        &self,
        receiver: Address,
        coefficient: f64,
    ) -> Result<String, EthErrors> {
        let value = self.get_claim_amount(coefficient);
        let tx = TransactionRequest::default()
            .with_to(receiver)
            .with_value(value);

        let tx_hash = self
            .sepolia_sender
            .send_transaction(tx)
            .await
            .map_err(|e| EthErrors::RpcError(e))?;
        Ok(format!("{}", tx_hash.tx_hash()))
    }

    async fn total_gas_fee(&self) -> Result<u128, EthErrors> {
        let gas_price = self
            .sepolia_sender
            .get_gas_price()
            .await
            .map_err(|e| EthErrors::RpcError(e))?;
        Ok(TX_GAS * gas_price)
    }

    pub async fn is_network_fee_ok(&self) -> Result<bool, EthErrors> {
        let tx_cost_limit = self.faucet_limit as f64 * self.fee_threshold;
        let total_gas_fee = self.total_gas_fee().await?;
        Ok(total_gas_fee < tx_cost_limit as u128)
    }
}
