use openvm_stark_sdk::{openvm_stark_backend::p3_field::PrimeField32, p3_baby_bear::BabyBear};
use snark_verifier_sdk::snark_verifier::halo2_base::halo2_proofs::halo2curves::bn256::Fr;

/// OpenVM's EXE and LEAF commitments are essentially 8 * Babybears each. During the generation of a
/// SNARK from the OpenVM RootProof, these commitments are compressed into a BN254 scalar.
pub fn compress_commitment(commitment: &[u32; 8]) -> Fr {
    let order = Fr::from(BabyBear::ORDER_U32 as u64);
    let mut base = Fr::one();
    let mut ret = Fr::zero();

    for v in commitment {
        ret += Fr::from(*v as u64) * base;
        base *= order;
    }

    ret
}
