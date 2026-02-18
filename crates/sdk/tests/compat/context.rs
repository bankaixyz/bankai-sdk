use std::env;

use anyhow::{Context, Result};
use bankai_sdk::{Bankai, HashingFunctionDto, Network};
use bankai_types::api::blocks::{
    BankaiBlockProofRequestDto, BankaiMmrProofRequestDto, BankaiTargetBlockSelectorDto,
    BlockStatusDto, LatestBlockQueryDto,
};
use bankai_types::api::ethereum::{
    BankaiBlockFilterDto, BankaiBlockSelectorDto, EthereumLightClientProofRequestDto,
    EthereumMmrProofRequestDto,
};
use bankai_types::api::proofs::ProofFormatDto;

#[derive(Debug, Clone)]
pub struct NamedFilterCase {
    pub label: String,
    pub filter: BankaiBlockFilterDto,
}

#[derive(Debug, Clone)]
pub struct NamedProofFormatCase {
    pub label: String,
    pub proof_format: ProofFormatDto,
}

#[derive(Debug, Clone)]
pub struct NamedTargetBlockCase {
    pub label: String,
    pub target_block: BankaiTargetBlockSelectorDto,
}

#[derive(Debug, Clone)]
pub struct NamedHashingCase {
    pub label: String,
    pub hashing_function: HashingFunctionDto,
}

pub struct CompatContext {
    pub api_base_url: String,
    pub sdk: Bankai,
    pub http: reqwest::Client,
}

impl CompatContext {
    pub fn from_env() -> Self {
        let api_base_url = env::var("COMPAT_API_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string())
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

    pub async fn filter_cases_core(&self) -> Result<Vec<NamedFilterCase>> {
        let latest_completed = self.latest_completed_height().await?;
        Ok(vec![
            NamedFilterCase {
                label: "selector=latest".to_string(),
                filter: BankaiBlockFilterDto::latest(),
            },
            NamedFilterCase {
                label: "selector=justified".to_string(),
                filter: BankaiBlockFilterDto::justified(),
            },
            NamedFilterCase {
                label: "selector=finalized".to_string(),
                filter: BankaiBlockFilterDto::finalized(),
            },
            NamedFilterCase {
                label: format!("bankai_block_number={latest_completed}"),
                filter: BankaiBlockFilterDto::with_bankai_block_number(latest_completed),
            },
        ])
    }

    pub async fn filter_cases_edge(&self) -> Result<Vec<NamedFilterCase>> {
        let latest_completed = self.latest_completed_height().await?;
        Ok(vec![NamedFilterCase {
            label: format!(
                "selector=finalized + bankai_block_number={latest_completed} (conflict)"
            ),
            filter: BankaiBlockFilterDto {
                selector: Some(BankaiBlockSelectorDto::Finalized),
                bankai_block_number: Some(latest_completed),
            },
        }])
    }

    pub fn proof_format_cases_core(&self) -> Vec<NamedProofFormatCase> {
        vec![
            NamedProofFormatCase {
                label: "proof_format=bin".to_string(),
                proof_format: ProofFormatDto::Bin,
            },
            NamedProofFormatCase {
                label: "proof_format=json".to_string(),
                proof_format: ProofFormatDto::Json,
            },
        ]
    }

    pub fn hashing_cases_core(&self) -> Vec<NamedHashingCase> {
        vec![NamedHashingCase {
            label: "hashing_function=keccak".to_string(),
            hashing_function: HashingFunctionDto::Keccak,
        }]
    }

    pub fn hashing_cases_edge(&self) -> Vec<NamedHashingCase> {
        vec![NamedHashingCase {
            label: "hashing_function=poseidon".to_string(),
            hashing_function: HashingFunctionDto::Poseidon,
        }]
    }

    pub async fn resolved_reference_block_for_filter(
        &self,
        filter: &BankaiBlockFilterDto,
    ) -> Result<u64> {
        if let Some(bankai_block_number) = filter.bankai_block_number {
            return Ok(bankai_block_number);
        }

        let epoch = self
            .sdk
            .api
            .ethereum()
            .epoch(filter)
            .await
            .context("failed to resolve reference block for filter via ethereum epoch")?;
        Ok(epoch.block_number)
    }

    pub async fn target_block_cases_core_for_filter(
        &self,
        filter: &BankaiBlockFilterDto,
    ) -> Result<Vec<NamedTargetBlockCase>> {
        let reference_block = self.resolved_reference_block_for_filter(filter).await?;
        let target_block = reference_block.checked_sub(1).context(
            "compat proof tests need at least two completed blocks (target must be lower than reference)",
        )?;

        let number_selector = BankaiTargetBlockSelectorDto {
            block_number: Some(target_block),
            block_hash: None,
        };

        let hash_from_number = self
            .sdk
            .api
            .blocks()
            .mmr_proof(&BankaiMmrProofRequestDto {
                filter: filter.clone(),
                target_block: number_selector.clone(),
                hashing_function: HashingFunctionDto::Keccak,
            })
            .await
            .context("failed to derive target block hash from mmr_proof")?
            .block_hash;

        Ok(vec![
            NamedTargetBlockCase {
                label: format!("target_block.block_number={target_block}"),
                target_block: number_selector,
            },
            NamedTargetBlockCase {
                label: "target_block.block_hash=<resolved_hash>".to_string(),
                target_block: BankaiTargetBlockSelectorDto {
                    block_number: None,
                    block_hash: Some(hash_from_number),
                },
            },
        ])
    }

    pub async fn target_block_cases_edge_for_filter(
        &self,
        filter: &BankaiBlockFilterDto,
    ) -> Result<Vec<NamedTargetBlockCase>> {
        let mut core = self.target_block_cases_core_for_filter(filter).await?;
        let by_number = core
            .iter()
            .find_map(|c| c.target_block.block_number)
            .context("missing target block number case")?;
        let by_hash = core
            .iter()
            .find_map(|c| c.target_block.block_hash.clone())
            .context("missing target block hash case")?;
        core.push(NamedTargetBlockCase {
            label: "target_block.block_number + target_block.block_hash (conflict)".to_string(),
            target_block: BankaiTargetBlockSelectorDto {
                block_number: Some(by_number),
                block_hash: Some(by_hash),
            },
        });
        Ok(core.into_iter().skip(2).collect())
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

    pub async fn execution_snapshot_for_filter(
        &self,
        filter: &BankaiBlockFilterDto,
    ) -> Result<bankai_types::api::ethereum::ExecutionSnapshotDto> {
        self.sdk
            .api
            .ethereum()
            .execution()
            .snapshot(filter)
            .await
            .context("failed to fetch execution snapshot")
    }

    pub async fn beacon_snapshot_for_filter(
        &self,
        filter: &BankaiBlockFilterDto,
    ) -> Result<bankai_types::api::ethereum::BeaconSnapshotDto> {
        self.sdk
            .api
            .ethereum()
            .beacon()
            .snapshot(filter)
            .await
            .context("failed to fetch beacon snapshot")
    }

    pub async fn execution_mmr_proof_request_for(
        &self,
        filter: &BankaiBlockFilterDto,
        hashing_function: HashingFunctionDto,
    ) -> Result<EthereumMmrProofRequestDto> {
        let snapshot = self.execution_snapshot_for_filter(filter).await?;
        Ok(EthereumMmrProofRequestDto {
            filter: filter.clone(),
            hashing_function,
            header_hash: snapshot.header_hash,
        })
    }

    pub async fn beacon_mmr_proof_request_for(
        &self,
        filter: &BankaiBlockFilterDto,
        hashing_function: HashingFunctionDto,
    ) -> Result<EthereumMmrProofRequestDto> {
        let snapshot = self.beacon_snapshot_for_filter(filter).await?;
        Ok(EthereumMmrProofRequestDto {
            filter: filter.clone(),
            hashing_function,
            header_hash: snapshot.beacon_root,
        })
    }

    pub async fn execution_light_client_request_for(
        &self,
        filter: &BankaiBlockFilterDto,
        hashing_function: HashingFunctionDto,
        proof_format: ProofFormatDto,
    ) -> Result<EthereumLightClientProofRequestDto> {
        let snapshot = self.execution_snapshot_for_filter(filter).await?;
        Ok(EthereumLightClientProofRequestDto {
            filter: filter.clone(),
            hashing_function,
            header_hashes: vec![snapshot.header_hash],
            proof_format,
        })
    }

    pub async fn beacon_light_client_request_for(
        &self,
        filter: &BankaiBlockFilterDto,
        hashing_function: HashingFunctionDto,
        proof_format: ProofFormatDto,
    ) -> Result<EthereumLightClientProofRequestDto> {
        let snapshot = self.beacon_snapshot_for_filter(filter).await?;
        Ok(EthereumLightClientProofRequestDto {
            filter: filter.clone(),
            hashing_function,
            header_hashes: vec![snapshot.beacon_root],
            proof_format,
        })
    }

    pub fn bankai_mmr_request_for(
        &self,
        filter: BankaiBlockFilterDto,
        target_block: BankaiTargetBlockSelectorDto,
        hashing_function: HashingFunctionDto,
    ) -> BankaiMmrProofRequestDto {
        BankaiMmrProofRequestDto {
            filter,
            target_block,
            hashing_function,
        }
    }

    pub fn bankai_block_proof_request_for(
        &self,
        filter: BankaiBlockFilterDto,
        target_block: BankaiTargetBlockSelectorDto,
        hashing_function: HashingFunctionDto,
        proof_format: ProofFormatDto,
    ) -> BankaiBlockProofRequestDto {
        BankaiBlockProofRequestDto {
            filter,
            target_block,
            hashing_function,
            proof_format,
        }
    }

    pub async fn execution_mmr_proof_request(&self) -> Result<EthereumMmrProofRequestDto> {
        self.execution_mmr_proof_request_for(&self.finalized_filter(), HashingFunctionDto::Keccak)
            .await
    }

    pub async fn beacon_mmr_proof_request(&self) -> Result<EthereumMmrProofRequestDto> {
        self.beacon_mmr_proof_request_for(&self.finalized_filter(), HashingFunctionDto::Keccak)
            .await
    }

    pub async fn execution_light_client_request(
        &self,
    ) -> Result<EthereumLightClientProofRequestDto> {
        self.execution_light_client_request_for(
            &self.finalized_filter(),
            HashingFunctionDto::Keccak,
            ProofFormatDto::Bin,
        )
        .await
    }

    pub async fn beacon_light_client_request(&self) -> Result<EthereumLightClientProofRequestDto> {
        self.beacon_light_client_request_for(
            &self.finalized_filter(),
            HashingFunctionDto::Keccak,
            ProofFormatDto::Bin,
        )
        .await
    }

    pub async fn bankai_mmr_request_from_latest(&self) -> Result<BankaiMmrProofRequestDto> {
        let (reference_block, target_block) = self.reference_and_target_block_numbers().await?;
        Ok(self.bankai_mmr_request_for(
            BankaiBlockFilterDto::with_bankai_block_number(reference_block),
            BankaiTargetBlockSelectorDto {
                block_number: Some(target_block),
                block_hash: None,
            },
            HashingFunctionDto::Keccak,
        ))
    }

    pub async fn conflicting_filter(&self) -> Result<BankaiBlockFilterDto> {
        let latest_completed = self.latest_completed_height().await?;
        Ok(BankaiBlockFilterDto {
            selector: Some(BankaiBlockSelectorDto::Finalized),
            bankai_block_number: Some(latest_completed),
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
