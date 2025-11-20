use openvm_native_recursion::halo2::utils::Halo2ParamsReader;
use openvm_sdk::Sdk;

/// Generate and return the EVM PLONK verifier's initcode.
pub(crate) fn generate() -> eyre::Result<Vec<u8>> {
    // Get the OpenVM SDK.
    let sdk = Sdk::riscv32();

    // Get the standard Halo2 proving key from the SDK.
    let halo2_pk = sdk.halo2_pk();

    // Read the Halo2 parameters for the default degree (k).
    let halo2_params = sdk
        .halo2_params_reader()
        .read_params(halo2_pk.wrapper.pinning.metadata.config_params.k);

    // Generate and return the verifier's bytecode.
    Ok(snark_verifier_sdk::evm::gen_evm_verifier_shplonk::<
        snark_verifier_sdk::halo2::aggregation::AggregationCircuit,
    >(
        &halo2_params,
        halo2_pk.wrapper.pinning.pk.get_vk(),
        halo2_pk.wrapper.pinning.metadata.num_pvs.clone(),
        None,
    ))
}
