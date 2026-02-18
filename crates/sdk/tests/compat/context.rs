use std::env;

use anyhow::{Context, Result};
use bankai_sdk::{Bankai, HashingFunctionDto, Network};
use bankai_types::api::blocks::{
    BankaiBlockProofRequestDto, BankaiMmrProofRequestDto, BankaiTargetBlockSelectorDto,
    BlockStatusDto, LatestBlockQueryDto,
};
use bankai_types::api::ethereum::{
    BankaiBlockFilterDto, EthereumLightClientProofRequestDto, EthereumMmrProofRequestDto,
};
use bankai_types::api::proofs::ProofFormatDto;

pub struct CompatContext {
    pub api_base_url: String,
    pub sdk: Bankai,
    pub http: reqwest::Client,
}

impl CompatContext {
    pub fn from_env() -> Self {
        let api_base_url = env::var("COMPAT_API_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
            .trim_end_matches('/')
            .to_string();

        Self {
            sdk: Bankai::new_with_base_url(Network::Local, api_base_url.clone(), None, None),
            api_base_url,
            http: reqwest::Client::new(),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base_url, path)
    }

    pub fn finalized_filter(&self) -> BankaiBlockFilterDto {
        BankaiBlockFilterDto::finalized()
    }

    pub async fn latest_completed_height(&self) -> Result<u64> {
        let query = LatestBlockQueryDto {
            status: Some(BlockStatusDto::Completed),
        };
        let latest = self
            .sdk
            .api
            .blocks()
            .latest(&query)
            .await
            .context("failed to fetch latest completed block")?;
        Ok(latest.height)
    }

    pub async fn epoch_from_finalized(
        &self,
    ) -> Result<bankai_types::api::ethereum::EthereumEpochDto> {
        let epoch = self
            .sdk
            .api
            .ethereum()
            .epoch(&self.finalized_filter())
            .await
            .context("failed to fetch finalized ethereum epoch")?;
        Ok(epoch)
    }

    pub async fn execution_header_hash_from_snapshot(&self) -> Result<String> {
        let snapshot = self
            .sdk
            .api
            .ethereum()
            .execution()
            .snapshot(&self.finalized_filter())
            .await
            .context("failed to fetch execution snapshot")?;
        Ok(snapshot.header_hash)
    }

    pub async fn beacon_header_hash_from_snapshot(&self) -> Result<String> {
        let snapshot = self
            .sdk
            .api
            .ethereum()
            .beacon()
            .snapshot(&self.finalized_filter())
            .await
            .context("failed to fetch beacon snapshot")?;
        Ok(snapshot.beacon_root)
    }

    pub async fn execution_mmr_proof_request(&self) -> Result<EthereumMmrProofRequestDto> {
        let header_hash = self.execution_header_hash_from_snapshot().await?;
        Ok(EthereumMmrProofRequestDto {
            filter: self.finalized_filter(),
            hashing_function: HashingFunctionDto::Keccak,
            header_hash,
        })
    }

    pub async fn beacon_mmr_proof_request(&self) -> Result<EthereumMmrProofRequestDto> {
        let header_hash = self.beacon_header_hash_from_snapshot().await?;
        Ok(EthereumMmrProofRequestDto {
            filter: self.finalized_filter(),
            hashing_function: HashingFunctionDto::Keccak,
            header_hash,
        })
    }

    pub async fn execution_light_client_request(
        &self,
    ) -> Result<EthereumLightClientProofRequestDto> {
        let header_hash = self.execution_header_hash_from_snapshot().await?;
        Ok(EthereumLightClientProofRequestDto {
            filter: self.finalized_filter(),
            hashing_function: HashingFunctionDto::Keccak,
            header_hashes: vec![header_hash],
            proof_format: ProofFormatDto::Bin,
        })
    }

    pub async fn beacon_light_client_request(&self) -> Result<EthereumLightClientProofRequestDto> {
        let header_hash = self.beacon_header_hash_from_snapshot().await?;
        Ok(EthereumLightClientProofRequestDto {
            filter: self.finalized_filter(),
            hashing_function: HashingFunctionDto::Keccak,
            header_hashes: vec![header_hash],
            proof_format: ProofFormatDto::Bin,
        })
    }

    pub async fn raw_bankai_mmr_request_json(&self) -> Result<serde_json::Value> {
        let req = self.bankai_mmr_request_from_latest().await?;
        serde_json::to_value(req).context("failed to serialize bankai mmr request")
    }

    pub async fn raw_bankai_block_proof_request_json(&self) -> Result<serde_json::Value> {
        let req = self.bankai_block_proof_request_from_latest().await?;
        serde_json::to_value(req).context("failed to serialize bankai block proof request")
    }

    pub async fn bankai_mmr_request_from_latest(&self) -> Result<BankaiMmrProofRequestDto> {
        let (reference_block, target_block) = self.reference_and_target_block_numbers().await?;
        Ok(BankaiMmrProofRequestDto {
            filter: BankaiBlockFilterDto::with_bankai_block_number(reference_block),
            target_block: BankaiTargetBlockSelectorDto {
                block_number: Some(target_block),
                block_hash: None,
            },
            hashing_function: HashingFunctionDto::Keccak,
        })
    }

    pub async fn bankai_block_proof_request_from_latest(
        &self,
    ) -> Result<BankaiBlockProofRequestDto> {
        let (reference_block, target_block) = self.reference_and_target_block_numbers().await?;
        Ok(BankaiBlockProofRequestDto {
            filter: BankaiBlockFilterDto::with_bankai_block_number(reference_block),
            target_block: BankaiTargetBlockSelectorDto {
                block_number: Some(target_block),
                block_hash: None,
            },
            hashing_function: HashingFunctionDto::Keccak,
            proof_format: ProofFormatDto::Bin,
        })
    }

    async fn reference_and_target_block_numbers(&self) -> Result<(u64, u64)> {
        let reference_block = self.latest_completed_height().await?;
        let target_block = reference_block.checked_sub(1).context(
            "compat proof tests need at least two completed blocks (target must be lower than reference)",
        )?;
        Ok((reference_block, target_block))
    }
}
