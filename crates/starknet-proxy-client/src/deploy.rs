use std::sync::Arc;
use ethers::abi::Tokenize;
use ethers::contract::ContractError;
use ethers::prelude::{ContractFactory, ContractInstance};
use ethers::providers::ProviderError;
use ethers::types::Bytes;
use ethers::utils::hex::{self, FromHex};
use utils::LocalWalletSignerMiddleware;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("['bytecode']['object'] is not a string")]
    BytecodeObject,
    #[error(transparent)]
    Hex(#[from] hex::FromHexError),
    #[error("Failed to parse URL")]
    UrlParser,
    #[error(transparent)]
    EthersContract(#[from] ContractError<LocalWalletSignerMiddleware>),
    #[error(transparent)]
    EthersProvider(#[from] ProviderError),
    #[error("Invalid contract build artifacts: missing field `{0}`")]
    ContractBuildArtifacts(&'static str),
}

const UNSAFE_PROXY: &str = include_str!("./artifacts/UnsafeProxy.json");

pub async fn deploy_contract<T: Tokenize>(
    client: Arc<LocalWalletSignerMiddleware>,
    contract_build_artifacts: &str,
    contructor_args: T,
) -> Result<ContractInstance<Arc<LocalWalletSignerMiddleware>, LocalWalletSignerMiddleware>, Error>
{
    let (abi, bytecode) = {
        let mut artifacts: serde_json::Value = serde_json::from_str(contract_build_artifacts)?;
        let abi_value = artifacts
            .get_mut("abi")
            .ok_or_else(|| Error::ContractBuildArtifacts("abi"))?
            .take();
        let bytecode_value = artifacts
            .get_mut("bytecode")
            .ok_or_else(|| Error::ContractBuildArtifacts("bytecode"))?
            .get_mut("object")
            .ok_or_else(|| Error::ContractBuildArtifacts("bytecode.object"))?
            .take();

        let abi = serde_json::from_value(abi_value)?;
        let bytecode = Bytes::from_hex(bytecode_value.as_str().ok_or(Error::BytecodeObject)?)?;
        (abi, bytecode)
    };

    let factory = ContractFactory::new(abi, bytecode, client.clone());

    Ok(factory
        .deploy(contructor_args)
        .map_err(Into::<ContractError<LocalWalletSignerMiddleware>>::into)?
        .send()
        .await
        .map_err(Into::<ContractError<LocalWalletSignerMiddleware>>::into)?)
}


/// Deploys new unsafe proxy contract:
///     - Implementation can be set only once at initialization
///     - Traditional (Safe) proxies can be upgraded multiple times
pub async fn deploy_contract_behind_unsafe_proxy<T: Tokenize>(
    client: Arc<LocalWalletSignerMiddleware>,
    contract_path: &str,
    constructor_args: T,
) -> Result<ContractInstance<Arc<LocalWalletSignerMiddleware>, LocalWalletSignerMiddleware>, Error> {
    let contract = deploy_contract(client.clone(), contract_path, constructor_args).await?;

    let proxy_contract =
        deploy_contract(client.clone(), UNSAFE_PROXY, contract.address()).await?;

    return Ok(proxy_contract);
}