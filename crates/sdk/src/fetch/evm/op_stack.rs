use alloy_primitives::{Address, FixedBytes, U256};
use alloy_rpc_types_eth::{Account as AlloyAccount, EIP1186AccountProofResponse};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use bankai_types::api::op_stack::{OpChainSnapshotSummaryDto, OpStackLightClientProofRequestDto};
use bankai_types::block::OpChainClient;
use bankai_types::common::HashingFunction;
use bankai_types::inputs::evm::{
    execution::{ReceiptProof, StorageSlotEntry, StorageSlotProof, TxProof},
    op_stack::OpStackHeaderProof,
};

use crate::errors::{SdkError, SdkResult};
use crate::fetch::{api::ApiClient, clients::op_stack_client::OpStackFetcher};

pub struct OpStackChainFetcher {
    api_client: ApiClient,
    chain_name: String,
    op_stack_client: OpStackFetcher,
}

impl OpStackChainFetcher {
    pub fn new(api_client: ApiClient, chain_name: String, rpc_url: String) -> Self {
        Self {
            api_client,
            chain_name,
            op_stack_client: OpStackFetcher::new(rpc_url),
        }
    }

    pub fn chain_name(&self) -> &str {
        &self.chain_name
    }

    pub async fn header(
        &self,
        block_number: u64,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpStackHeaderProof> {
        let snapshot = self.validated_snapshot(filter.clone()).await?;
        let header = self.header_only(block_number).await?;
        self.header_from_execution_header(header, snapshot, hashing_function, filter)
            .await
    }

    pub async fn header_by_hash(
        &self,
        header_hash: FixedBytes<32>,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpStackHeaderProof> {
        let snapshot = self.validated_snapshot(filter.clone()).await?;
        let header = self.header_only_by_hash(header_hash).await?;
        self.header_from_execution_header(header, snapshot, hashing_function, filter)
            .await
    }

    pub async fn header_only(&self, block_number: u64) -> SdkResult<alloy_rpc_types_eth::Header> {
        self.op_stack_client.fetch_header(block_number).await
    }

    pub async fn header_only_by_hash(
        &self,
        header_hash: FixedBytes<32>,
    ) -> SdkResult<alloy_rpc_types_eth::Header> {
        self.op_stack_client.fetch_header_by_hash(header_hash).await
    }

    pub async fn account(
        &self,
        block_number: u64,
        address: Address,
    ) -> SdkResult<EIP1186AccountProofResponse> {
        self.op_stack_client
            .fetch_account_proof(address, block_number)
            .await
    }

    pub async fn tx_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<TxProof> {
        let network_id = self.chain_id().await?;
        self.op_stack_client
            .fetch_tx_proof(tx_hash, network_id)
            .await
    }

    pub async fn receipt_proof(&self, tx_hash: FixedBytes<32>) -> SdkResult<ReceiptProof> {
        let network_id = self.chain_id().await?;
        self.op_stack_client
            .fetch_receipt_proof(tx_hash, network_id)
            .await
    }

    pub async fn storage_slot_proof(
        &self,
        block_number: u64,
        address: Address,
        slot_keys: &[U256],
    ) -> SdkResult<StorageSlotProof> {
        let proof = self
            .op_stack_client
            .fetch_storage_slot_proof(address, block_number, slot_keys)
            .await?;
        let header = self.header_only(block_number).await?;
        let network_id = self.chain_id().await?;

        let slots = proof
            .storage_proof
            .into_iter()
            .map(|slot| StorageSlotEntry {
                slot_key: slot.key.as_b256().into(),
                slot_value: slot.value,
                storage_mpt_proof: slot.proof,
            })
            .collect();

        let account = AlloyAccount {
            balance: proof.balance,
            nonce: proof.nonce,
            code_hash: proof.code_hash,
            storage_root: proof.storage_hash,
        };

        Ok(StorageSlotProof {
            account,
            address,
            network_id,
            block_number,
            state_root: header.state_root,
            account_mpt_proof: proof.account_proof,
            slots,
        })
    }

    pub async fn chain_id(&self) -> SdkResult<u64> {
        self.op_stack_client.fetch_chain_id().await
    }

    pub async fn snapshot(
        &self,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpChainSnapshotSummaryDto> {
        self.api_client
            .op_stack()
            .snapshot(&self.chain_name, &filter)
            .await
    }

    async fn header_from_execution_header(
        &self,
        header: alloy_rpc_types_eth::Header,
        snapshot: OpChainSnapshotSummaryDto,
        hashing_function: HashingFunction,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpStackHeaderProof> {
        let request = OpStackLightClientProofRequestDto {
            filter,
            hashing_function,
            header_hashes: vec![header.hash.to_string()],
            proof_format: bankai_types::common::ProofFormat::Bin,
        };
        let proof = self
            .api_client
            .op_stack()
            .light_client_proof(&self.chain_name, &request)
            .await?;
        let mmr_proof = proof.mmr_proofs.into_iter().next().ok_or_else(|| {
            SdkError::NotFound(format!(
                "missing OP MMR proof for chain {}",
                self.chain_name
            ))
        })?;

        Ok(OpStackHeaderProof {
            header,
            snapshot: snapshot_to_witness(snapshot)?,
            merkle_proof: proof.merkle_proof.into(),
            mmr_proof: mmr_proof.into(),
        })
    }

    async fn validated_snapshot(
        &self,
        filter: BankaiBlockFilterDto,
    ) -> SdkResult<OpChainSnapshotSummaryDto> {
        let snapshot = self.snapshot(filter).await?;
        let rpc_chain_id = self.chain_id().await?;
        if rpc_chain_id != snapshot.chain_id {
            return Err(SdkError::InvalidInput(format!(
                "OP chain_id mismatch for {}: rpc returned {}, snapshot returned {}",
                self.chain_name, rpc_chain_id, snapshot.chain_id
            )));
        }
        Ok(snapshot)
    }
}

fn snapshot_to_witness(snapshot: OpChainSnapshotSummaryDto) -> SdkResult<OpChainClient> {
    Ok(OpChainClient {
        chain_id: snapshot.chain_id,
        block_number: snapshot.end_height,
        header_hash: snapshot
            .header_hash
            .parse()
            .map_err(|e| SdkError::InvalidInput(format!("invalid OP snapshot header hash: {e}")))?,
        l1_submission_block: snapshot.l1_submission_block,
        mmr_root_keccak: snapshot
            .mmr_roots
            .keccak_root
            .parse()
            .map_err(|e| SdkError::InvalidInput(format!("invalid OP snapshot keccak root: {e}")))?,
        mmr_root_poseidon: snapshot.mmr_roots.poseidon_root.parse().map_err(|e| {
            SdkError::InvalidInput(format!("invalid OP snapshot poseidon root: {e}"))
        })?,
    })
}
