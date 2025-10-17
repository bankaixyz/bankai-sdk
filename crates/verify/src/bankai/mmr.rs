// extern crate alloc;
// use alloc::string::String;
// use alloc::sync::Arc;
// use alloc::vec::Vec;

// use accumulators::store::memory::InMemoryStore;
use accumulators::{
    hasher::{keccak::KeccakHasher, stark_poseidon::StarkPoseidonHasher, Hasher},
    mmr::{Proof, ProofOptions, MMR},
};
// use bankai_types::{
//     proofs::{HashingFunctionDto, MmrProofDto},
//     utils::mmr::hash_to_leaf,
// };

// use crate::VerifyError;

// pub struct BankaiMmr;

// impl BankaiMmr {
//     pub fn mmr_from_peaks(
//         peaks_hashes: Vec<String>,
//         elements_count: usize,
//         hashing_function: HashingFunctionDto,
//     ) -> Result<MMR, VerifyError> {
//         let hasher: Arc<dyn Hasher> = match hashing_function {
//             HashingFunctionDto::Keccak => Arc::new(KeccakHasher::new()),
//             HashingFunctionDto::Poseidon => Arc::new(StarkPoseidonHasher::new(Some(true))),
//         };
//         let store = Arc::new(InMemoryStore::default());
//         let mmr = MMR::create_from_peaks_sync(
//             store.clone(),
//             hasher.clone(),
//             None,
//             peaks_hashes,
//             elements_count,
//         )
//         .map_err(|_| VerifyError::InvalidMmrTree)?;

//         Ok(mmr)
//     }

//     pub fn verify_mmr_proof(proof: MmrProofDto) -> Result<bool, VerifyError> {
//         let mmr = Self::mmr_from_peaks(
//             proof.peaks.clone(),
//             proof.elements_count as usize,
//             proof.hashing_function,
//         )?;

//         let element_hash = hash_to_leaf(proof.header_hash, &proof.hashing_function.clone());
//         let proof_type = Proof {
//             element_index: proof.elements_index as usize,
//             element_hash,
//             siblings_hashes: proof.path,
//             elements_count: proof.elements_count as usize,
//             peaks_hashes: proof.peaks,
//         };

//         let options = ProofOptions {
//             elements_count: Some(proof.elements_count as usize),
//             formatting_opts: None,
//         };

//         mmr.verify_proof_sync(proof_type.clone(), proof_type.element_hash, Some(options))
//             .map_err(|_| VerifyError::InvalidMmrProof)?;

//         // Ok(true)
//         Ok(true)
//     }
// }
