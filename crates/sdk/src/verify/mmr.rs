use std::{sync::Arc};

use accumulators::{hasher::{keccak::KeccakHasher, stark_poseidon::StarkPoseidonHasher, Hasher}, mmr::{Proof, ProofOptions, MMR}};
use accumulators::store::memory::InMemoryStore;
use alloy_primitives::{
    hex::{FromHex, ToHexExt}
};
use anyhow::Error;
use bankai_types::{api::{HashingFunctionDto, MmrProofDto}, utils::mmr::hash_to_leaf};



pub struct BankaiMmr {
    hasher: Arc<dyn Hasher>,
    mmr: MMR,
}



impl BankaiMmr {
    pub fn new(hashing_function: HashingFunctionDto) -> Self {
        let hasher: Arc<dyn Hasher> = match hashing_function {
            HashingFunctionDto::Keccak => Arc::new(KeccakHasher::new()),
            HashingFunctionDto::Poseidon => Arc::new(StarkPoseidonHasher::new(Some(true))),
        };
        let store = Arc::new(InMemoryStore::default());
        Self { mmr: MMR::new(store, hasher.clone(), None), hasher }
    }

    pub async fn verify_proof(self, proof: MmrProofDto) -> Result<bool, Error> {
        let element_hash = hash_to_leaf(proof.header_hash, &proof.hashing_function);
        let proof_type = Proof {
            element_index: proof.elements_index as usize,
            element_hash,
            siblings_hashes: proof
                .path
                .iter()
                .map(|h| format!("0x{}", h.encode_hex()))
                .collect(),
            elements_count: proof.elements_count as usize,
            peaks_hashes: proof
                .peaks
                .iter()
                .map(|h| format!("0x{}", h.encode_hex()))
                .collect(),
        };

        let options = ProofOptions {
            elements_count: Some(proof.elements_count as usize),
            formatting_opts: None,
        };

        self.mmr
            .verify_proof(proof_type.clone(), proof_type.element_hash, Some(options))
            .await
            .map_err(Error::from)
    }


    
    
}