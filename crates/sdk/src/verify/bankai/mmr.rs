use std::sync::Arc;

use accumulators::store::memory::InMemoryStore;
use accumulators::{
    hasher::{keccak::KeccakHasher, stark_poseidon::StarkPoseidonHasher, Hasher},
    mmr::{Proof, ProofOptions, MMR},
};
use bankai_types::{
    api::proofs::{HashingFunctionDto, MmrProofDto},
    utils::mmr::hash_to_leaf,
};
use crate::errors::{SdkError, SdkResult};

pub struct BankaiMmr;

impl BankaiMmr {
    pub async fn mmr_from_peaks(
        peaks_hashes: Vec<String>,
        elements_count: usize,
        hashing_function: HashingFunctionDto,
    ) -> SdkResult<MMR> {
        let hasher: Arc<dyn Hasher> = match hashing_function {
            HashingFunctionDto::Keccak => Arc::new(KeccakHasher::new()),
            HashingFunctionDto::Poseidon => Arc::new(StarkPoseidonHasher::new(Some(true))),
        };
        let store = Arc::new(InMemoryStore::default());
        let mmr = MMR::create_from_peaks(
            store.clone(),
            hasher.clone(),
            None,
            peaks_hashes,
            elements_count,
        )
        .await
        .map_err(|e| SdkError::Verification(format!("failed to create mmr: {e}")))?;

        Ok(mmr)
    }

    pub async fn verify_mmr_proof(proof: MmrProofDto) -> SdkResult<bool> {
        let mmr = Self::mmr_from_peaks(
            proof.peaks.clone(),
            proof.elements_count as usize,
            proof.hashing_function.clone(),
        )
        .await?;

        let element_hash = hash_to_leaf(proof.header_hash, &proof.hashing_function.clone());
        let proof_type = Proof {
            element_index: proof.elements_index as usize,
            element_hash,
            siblings_hashes: proof.path,
            elements_count: proof.elements_count as usize,
            peaks_hashes: proof.peaks,
        };

        let options = ProofOptions {
            elements_count: Some(proof.elements_count as usize),
            formatting_opts: None,
        };

        mmr.verify_proof(proof_type.clone(), proof_type.element_hash, Some(options))
            .await
            .map_err(|e| SdkError::Verification(format!("mmr verify failed: {e}")))
    }
}
