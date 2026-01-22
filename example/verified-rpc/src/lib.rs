use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};

use alloy_primitives::{BlockHash, FixedBytes, hex};
use alloy_provider::{Provider, RootProvider, network::Ethereum};
use alloy_rpc_types_eth::Header as RpcHeader;
use alloy_rpc_types_eth::BlockNumberOrTag;
use bankai_sdk::{ApiClient, HashingFunctionDto, Network};
use bankai_sdk::errors::SdkError;
use bankai_types::api::blocks::{BlockStatusDto, BlockSummaryDto, LatestBlockQueryDto};
use bankai_types::api::error::ErrorResponse;
use bankai_types::api::proofs::{BankaiBlockProofDto, MmrProofDto, MmrProofRequestDto};
use bankai_types::block::BankaiBlock;
use bankai_types::fetch::evm::{MmrProof};
use bankai_types::fetch::evm::execution::ExecutionHeaderProof;
use bankai_verify::VerifyError;
use bankai_verify::bankai::stwo::verify_stwo_proof;
use bankai_verify::evm::execution::ExecutionVerifier;
use cairo_air::CairoProof;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::Value;
use starknet_ff::FieldElement;
use stwo::core::vcs::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_serialize::deserialize::CairoDeserialize;
use thiserror::Error;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Debug, Error)]
pub enum VerifiedRpcError {
    #[error("rpc transport error: {0}")]
    RpcTransport(String),
    #[error("rpc error {code}: {message}")]
    Rpc { code: i64, message: String },
    #[error("rpc response missing result for {method}")]
    RpcMissingResult { method: String },
    #[error("rpc response decode error: {0}")]
    RpcDecode(#[from] serde_json::Error),
    #[error("bankai api error: {0}")]
    BankaiApi(#[from] SdkError),
    #[error("bankai api transport error: {0}")]
    BankaiTransport(String),
    #[error("bankai api http error (status {status}): {body}")]
    BankaiHttp { status: u16, body: String },
    #[error("bankai api error response: {code} - {message}")]
    BankaiApiResponse {
        code: String,
        message: String,
        error_id: String,
    },
    #[error("mmr proof not found for header hash {0}")]
    MmrProofMissing(String),
    #[error("header hash mismatch: expected {expected}, computed {computed}")]
    HeaderHashMismatch { expected: String, computed: String },
    #[error("block proof parse error: {0}")]
    BlockProofParse(String),
    #[error("proof verification failed: {0}")]
    ProofVerification(VerifyError),
}

#[derive(Debug, Clone)]
pub struct VerifiedHeader {
    pub header: RpcHeader,
    pub header_hash: FixedBytes<32>,
    pub bankai_block_number: u64,
    pub mmr_root: FixedBytes<32>,
    pub mmr_proof: MmrProof,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

pub trait JsonRpcTransport {
    fn send<'a>(&'a self, request: JsonRpcRequest) -> BoxFuture<'a, Result<Value, VerifiedRpcError>>;
}

pub struct VerifiedRpcClient<T> {
    hashing_function: HashingFunctionDto,
    execution_network_id: u64,
    bankai_api: BankaiApiClient,
    transport: T,
    request_id: AtomicU64,
}

impl<T> VerifiedRpcClient<T>
where
    T: JsonRpcTransport,
{
    pub fn with_transport(
        network: Network,
        transport: T,
        bankai_api: BankaiApiClient,
    ) -> Self {
        Self {
            hashing_function: HashingFunctionDto::Keccak,
            execution_network_id: network.execution_network_id(),
            bankai_api,
            transport,
            request_id: AtomicU64::new(1),
        }
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value, VerifiedRpcError> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: self.next_request_id(),
            method: method.to_string(),
            params,
        };
        let response_value = self.transport.send(request).await?;
        let response: JsonRpcResponse<Value> = serde_json::from_value(response_value)?;
        if let Some(error) = response.error {
            return Err(VerifiedRpcError::Rpc {
                code: error.code,
                message: error.message,
            });
        }
        let result = response
            .result
            .filter(|value| !value.is_null())
            .ok_or_else(|| VerifiedRpcError::RpcMissingResult {
                method: method.to_string(),
            })?;
        Ok(result)
    }

    pub async fn get_block_by_number_verified(
        &self,
        block_number: u64,
        bankai_block_number: Option<u64>,
    ) -> Result<VerifiedHeader, VerifiedRpcError> {
        let params = Value::Array(vec![Value::String(format_block_number(block_number)), Value::Bool(false)]);
        let header = self.fetch_header("eth_getBlockByNumber", params).await?;
        self.verify_header(header, bankai_block_number).await
    }

    pub async fn get_block_by_hash_verified(
        &self,
        block_hash: &str,
        bankai_block_number: Option<u64>,
    ) -> Result<VerifiedHeader, VerifiedRpcError> {
        let params = Value::Array(vec![Value::String(block_hash.to_string()), Value::Bool(false)]);
        let header = self.fetch_header("eth_getBlockByHash", params).await?;
        self.verify_header(header, bankai_block_number).await
    }

    async fn fetch_header(
        &self,
        method: &str,
        params: Value,
    ) -> Result<RpcHeader, VerifiedRpcError> {
        let result = self.call(method, params).await?;
        let header: RpcHeader = serde_json::from_value(result)?;
        Ok(header)
    }

    async fn verify_header(
        &self,
        header: RpcHeader,
        bankai_block_number: Option<u64>,
    ) -> Result<VerifiedHeader, VerifiedRpcError> {
        verify_execution_header(
            &self.bankai_api,
            self.hashing_function,
            self.execution_network_id,
            header,
            bankai_block_number,
        )
        .await
    }

    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed)
    }
}

pub struct VerifiedProvider<P> {
    inner: P,
    hashing_function: HashingFunctionDto,
    execution_network_id: u64,
    bankai_api: BankaiApiClient,
}

impl<P> VerifiedProvider<P>
where
    P: Provider<Ethereum>,
{
    pub fn new(network: Network, provider: P, bankai_api_base: Option<String>) -> Self {
        Self {
            inner: provider,
            hashing_function: HashingFunctionDto::Keccak,
            execution_network_id: network.execution_network_id(),
            bankai_api: BankaiApiClient::new(network, bankai_api_base),
        }
    }

    pub fn inner(&self) -> &P {
        &self.inner
    }

    pub fn into_inner(self) -> P {
        self.inner
    }

    pub async fn get_block_by_number_verified(
        &self,
        block_number: u64,
        bankai_block_number: Option<u64>,
    ) -> Result<VerifiedHeader, VerifiedRpcError> {
        let block = self
            .inner
            .get_block_by_number(BlockNumberOrTag::Number(block_number))
            .await
            .map_err(|err| VerifiedRpcError::RpcTransport(err.to_string()))?
            .ok_or_else(|| VerifiedRpcError::RpcMissingResult {
                method: "eth_getBlockByNumber".to_string(),
            })?;
        let header = block.header().clone();
        verify_execution_header(
            &self.bankai_api,
            self.hashing_function,
            self.execution_network_id,
            header,
            bankai_block_number,
        )
        .await
    }

    pub async fn get_block_by_hash_verified(
        &self,
        block_hash: BlockHash,
        bankai_block_number: Option<u64>,
    ) -> Result<VerifiedHeader, VerifiedRpcError> {
        let block = self
            .inner
            .get_block_by_hash(block_hash)
            .await
            .map_err(|err| VerifiedRpcError::RpcTransport(err.to_string()))?
            .ok_or_else(|| VerifiedRpcError::RpcMissingResult {
                method: "eth_getBlockByHash".to_string(),
            })?;
        let header = block.header().clone();
        verify_execution_header(
            &self.bankai_api,
            self.hashing_function,
            self.execution_network_id,
            header,
            bankai_block_number,
        )
        .await
    }
}

impl<P> Provider<Ethereum> for VerifiedProvider<P>
where
    P: Provider<Ethereum>,
{
    fn root(&self) -> &RootProvider<Ethereum> {
        self.inner.root()
    }
}

#[cfg(any(feature = "native", feature = "wasm"))]
pub struct ReqwestTransport {
    rpc_url: String,
    client: reqwest::Client,
}

#[cfg(any(feature = "native", feature = "wasm"))]
impl ReqwestTransport {
    pub fn new(rpc_url: String) -> Self {
        Self {
            rpc_url,
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(any(feature = "native", feature = "wasm"))]
impl JsonRpcTransport for ReqwestTransport {
    fn send<'a>(&'a self, request: JsonRpcRequest) -> BoxFuture<'a, Result<Value, VerifiedRpcError>> {
        Box::pin(async move {
            let response = self
                .client
                .post(&self.rpc_url)
                .json(&request)
                .send()
                .await
                .map_err(|err| VerifiedRpcError::RpcTransport(err.to_string()))?;
            let status = response.status();
            let payload = response
                .json::<Value>()
                .await
                .map_err(|err| VerifiedRpcError::RpcTransport(err.to_string()))?;
            if !status.is_success() {
                return Err(VerifiedRpcError::RpcTransport(format!(
                    "http status {}: {}",
                    status.as_u16(),
                    payload
                )));
            }
            Ok(payload)
        })
    }
}

#[cfg(any(feature = "native", feature = "wasm"))]
impl VerifiedRpcClient<ReqwestTransport> {
    pub fn new(
        network: Network,
        rpc_url: String,
        bankai_api_base: Option<String>,
    ) -> Self {
        let bankai_api = BankaiApiClient::new(network, bankai_api_base);
        let transport = ReqwestTransport::new(rpc_url);
        Self::with_transport(network, transport, bankai_api)
    }
}

pub enum BankaiApiClient {
    Sdk(ApiClient),
    #[cfg(any(feature = "native", feature = "wasm"))]
    Http(HttpBankaiApiClient),
}

impl BankaiApiClient {
    pub fn new(network: Network, bankai_api_base: Option<String>) -> Self {
        match bankai_api_base {
            Some(base_url) => {
                #[cfg(any(feature = "native", feature = "wasm"))]
                {
                    return BankaiApiClient::Http(HttpBankaiApiClient::new(base_url));
                }
                #[cfg(not(any(feature = "native", feature = "wasm")))]
                {
                    let _ = base_url;
                    BankaiApiClient::Sdk(ApiClient::new(network))
                }
            }
            None => BankaiApiClient::Sdk(ApiClient::new(network)),
        }
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, VerifiedRpcError> {
        match self {
            BankaiApiClient::Sdk(client) => Ok(client.get_latest_block_number().await?),
            #[cfg(any(feature = "native", feature = "wasm"))]
            BankaiApiClient::Http(client) => client.get_latest_block_number().await,
        }
    }

    pub async fn get_block_proof(
        &self,
        block_number: u64,
    ) -> Result<BankaiBlockProofDto, VerifiedRpcError> {
        match self {
            BankaiApiClient::Sdk(client) => Ok(client.get_block_proof(block_number).await?),
            #[cfg(any(feature = "native", feature = "wasm"))]
            BankaiApiClient::Http(client) => client.get_block_proof(block_number).await,
        }
    }

    pub async fn get_mmr_proof(
        &self,
        request: &MmrProofRequestDto,
    ) -> Result<MmrProofDto, VerifiedRpcError> {
        match self {
            BankaiApiClient::Sdk(client) => Ok(client.get_mmr_proof(request).await?),
            #[cfg(any(feature = "native", feature = "wasm"))]
            BankaiApiClient::Http(client) => client.get_mmr_proof(request).await,
        }
    }
}

#[cfg(any(feature = "native", feature = "wasm"))]
pub struct HttpBankaiApiClient {
    base_url: String,
    client: reqwest::Client,
}

#[cfg(any(feature = "native", feature = "wasm"))]
impl HttpBankaiApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, VerifiedRpcError> {
        let url = format!("{}/v1/blocks/latest", self.base_url);
        let query = LatestBlockQueryDto {
            status: Some(BlockStatusDto::Completed),
        };
        let response = self.client.get(&url).query(&query).send().await;
        let block_summary: BlockSummaryDto = self.handle_response(response).await?;
        Ok(block_summary.height)
    }

    pub async fn get_block_proof(
        &self,
        block_number: u64,
    ) -> Result<BankaiBlockProofDto, VerifiedRpcError> {
        let url = format!("{}/v1/proofs/block/{}", self.base_url, block_number);
        let response = self.client.get(&url).send().await;
        self.handle_response(response).await
    }

    pub async fn get_mmr_proof(
        &self,
        request: &MmrProofRequestDto,
    ) -> Result<MmrProofDto, VerifiedRpcError> {
        let url = format!("{}/v1/proofs/mmr", self.base_url);
        let response = self.client.post(&url).json(request).send().await;
        self.handle_response(response).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<T, VerifiedRpcError> {
        let response =
            response.map_err(|err| VerifiedRpcError::BankaiTransport(err.to_string()))?;
        if response.status().is_success() {
            let value = response
                .json::<T>()
                .await
                .map_err(|err| VerifiedRpcError::BankaiTransport(err.to_string()))?;
            return Ok(value);
        }

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        if let Ok(api_err) = serde_json::from_str::<ErrorResponse>(&body) {
            return Err(VerifiedRpcError::BankaiApiResponse {
                code: api_err.code,
                message: api_err.message,
                error_id: api_err.error_id,
            });
        }
        Err(VerifiedRpcError::BankaiHttp { status, body })
    }
}

fn parse_block_proof_value(
    value: Value,
) -> Result<CairoProof<Blake2sMerkleHasher>, VerifiedRpcError> {
    if let Ok(parsed) = serde_json::from_value::<CairoProof<Blake2sMerkleHasher>>(value.clone()) {
        return Ok(parsed);
    }
    let data = value
        .as_array()
        .ok_or_else(|| VerifiedRpcError::BlockProofParse("expected proof array".to_string()))?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| {
                    VerifiedRpcError::BlockProofParse("proof element is not a string".to_string())
                })?
                .parse::<FieldElement>()
                .map_err(|err| VerifiedRpcError::BlockProofParse(err.to_string()))
        })
        .collect::<Result<Vec<_>, VerifiedRpcError>>()?;
    let proof = <CairoProof<Blake2sMerkleHasher> as CairoDeserialize>::deserialize(&mut data.iter());
    Ok(proof)
}

fn execution_mmr_root(block: &BankaiBlock, hashing: HashingFunctionDto) -> FixedBytes<32> {
    match hashing {
        HashingFunctionDto::Keccak => block.execution.mmr_root_keccak,
        HashingFunctionDto::Poseidon => block.execution.mmr_root_poseidon,
    }
}

async fn verify_execution_header(
    bankai_api: &BankaiApiClient,
    hashing_function: HashingFunctionDto,
    execution_network_id: u64,
    header: RpcHeader,
    bankai_block_number: Option<u64>,
) -> Result<VerifiedHeader, VerifiedRpcError> {
    let bankai_block_number = resolve_bankai_block_number(bankai_api, bankai_block_number).await?;
    let header_hash = header.inner.hash_slow();
    let header_hash_hex = format_hash(header_hash);

    let mmr_proof = fetch_mmr_proof(
        bankai_api,
        execution_network_id,
        hashing_function,
        bankai_block_number,
        &header_hash_hex,
    )
    .await?;

    if mmr_proof.header_hash != header_hash {
        return Err(VerifiedRpcError::HeaderHashMismatch {
            expected: format_hash(mmr_proof.header_hash),
            computed: header_hash_hex,
        });
    }

    let bankai_block = fetch_and_verify_bankai_block(bankai_api, bankai_block_number).await?;
    let mmr_root = execution_mmr_root(&bankai_block, hashing_function);

    let proof = ExecutionHeaderProof {
        header: header.clone(),
        mmr_proof: mmr_proof.clone(),
    };
    ExecutionVerifier::verify_header_proof(&proof, mmr_root)
        .map_err(VerifiedRpcError::ProofVerification)?;

    Ok(VerifiedHeader {
        header,
        header_hash,
        bankai_block_number,
        mmr_root,
        mmr_proof,
    })
}

async fn resolve_bankai_block_number(
    bankai_api: &BankaiApiClient,
    bankai_block_number: Option<u64>,
) -> Result<u64, VerifiedRpcError> {
    match bankai_block_number {
        Some(number) => Ok(number),
        None => bankai_api.get_latest_block_number().await,
    }
}

async fn fetch_mmr_proof(
    bankai_api: &BankaiApiClient,
    execution_network_id: u64,
    hashing_function: HashingFunctionDto,
    bankai_block_number: u64,
    header_hash_hex: &str,
) -> Result<MmrProof, VerifiedRpcError> {
    let request = MmrProofRequestDto {
        network_id: execution_network_id,
        block_number: bankai_block_number,
        hashing_function,
        header_hash: header_hash_hex.to_string(),
    };
    let result = bankai_api.get_mmr_proof(&request).await;
    match result {
        Ok(proof) => Ok(proof.into()),
        Err(error) if is_missing_mmr_proof(&error) => {
            Err(VerifiedRpcError::MmrProofMissing(header_hash_hex.to_string()))
        }
        Err(error) => Err(error),
    }
}

async fn fetch_and_verify_bankai_block(
    bankai_api: &BankaiApiClient,
    bankai_block_number: u64,
) -> Result<BankaiBlock, VerifiedRpcError> {
    let block_proof = bankai_api.get_block_proof(bankai_block_number).await?;
    let proof = parse_block_proof_value(block_proof.proof)?;
    verify_stwo_proof(proof).map_err(VerifiedRpcError::ProofVerification)
}

fn format_block_number(block_number: u64) -> String {
    format!("0x{block_number:x}")
}

fn format_hash(hash: FixedBytes<32>) -> String {
    format!("0x{}", hex::encode(hash))
}

fn is_missing_mmr_proof(error: &VerifiedRpcError) -> bool {
    match error {
        VerifiedRpcError::BankaiApi(SdkError::NotFound(_)) => true,
        VerifiedRpcError::BankaiApi(SdkError::Api { status, .. }) => status.as_u16() == 404,
        VerifiedRpcError::BankaiApi(SdkError::ApiErrorResponse { code, .. }) => {
            code.contains("not_found")
        }
        VerifiedRpcError::BankaiHttp { status, .. } => *status == 404,
        VerifiedRpcError::BankaiApiResponse { code, .. } => code.contains("not_found"),
        _ => false,
    }
}
