extern crate alloc;

use alloy_primitives::FixedBytes;
use bankai_types::fetch::evm::beacon::BeaconHeaderProof;
use bankai_types::verify::evm::beacon::BeaconHeader;
use tree_hash::TreeHash;

use crate::bankai::mmr::MmrVerifier;
use crate::VerifyError;

/// Verifier for EVM beacon chain (consensus layer) proofs
///
/// Provides methods to verify beacon chain headers against trusted MMR roots.
/// Beacon headers contain consensus layer information including validator data,
/// randao values, and execution payload commitments.
pub struct BeaconVerifier;

impl BeaconVerifier {
    /// Verifies a beacon chain header using an MMR inclusion proof
    ///
    /// This method establishes trust in a beacon chain header by:
    /// 1. Verifying the MMR root matches the expected root from the STWO proof
    /// 2. Verifying the MMR inclusion proof
    /// 3. Verifying the header's tree hash root matches the value committed in the MMR
    ///
    /// Once verified, the beacon header can be trusted and used to verify consensus layer data.
    ///
    /// # Arguments
    ///
    /// * `proof` - The beacon header proof containing the header and MMR inclusion proof
    /// * `root` - The trusted MMR root from the verified STWO proof
    ///
    /// # Returns
    ///
    /// Returns the verified `BeaconHeader` containing all beacon chain data including:
    /// - Slot number
    /// - Proposer index
    /// - Parent root
    /// - State root
    /// - Body root (contains execution payload commitment)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `InvalidMmrRoot`: The MMR root in the proof doesn't match the expected root
    /// - `InvalidMmrProof`: The MMR inclusion proof is invalid
    /// - `InvalidHeaderHash`: The header's tree hash root doesn't match the MMR commitment
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bankai_verify::evm::BeaconVerifier;
    /// use bankai_types::fetch::evm::beacon::BeaconHeaderProof;
    /// use alloy_primitives::FixedBytes;
    ///
    /// # fn example(proof: BeaconHeaderProof, mmr_root: FixedBytes<32>) -> Result<(), Box<dyn std::error::Error>> {
    /// let verified_header = BeaconVerifier::verify_header_proof(&proof, mmr_root)?;
    /// println!("Verified beacon slot {}", verified_header.slot);
    /// println!("Proposer index: {}", verified_header.proposer_index);
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_header_proof(
        proof: &BeaconHeaderProof,
        root: FixedBytes<32>,
    ) -> Result<BeaconHeader, VerifyError> {
        if proof.mmr_proof.root != root {
            return Err(VerifyError::InvalidMmrRoot);
        }

        MmrVerifier::verify_mmr_proof(&proof.mmr_proof.clone())?;

        let hash = proof.header.tree_hash_root();
        if hash != proof.mmr_proof.header_hash {
            return Err(VerifyError::InvalidHeaderHash);
        }

        Ok(proof.header.clone())
    }
}
