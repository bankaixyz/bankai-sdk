use crate::api::proofs::MmrProofDto;
use alloy_primitives::{Address, Bytes, FixedBytes};
use alloy_rpc_types::{Account, Header as ExecutionHeader};

#[derive(Debug)]
pub struct ExecutionHeaderProof {
    pub header: ExecutionHeader,
    pub mmr_proof: MmrProofDto,
}

#[derive(Debug)]
    pub struct AccountProof {
    pub account: Account,
    pub address: Address,
    pub state_root: FixedBytes<32>,
    pub mpt_proof: Vec<Bytes>,
}