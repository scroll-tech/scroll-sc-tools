use openvm_native_recursion::halo2::utils::Halo2ParamsReader;
use openvm_sdk::Sdk;
use snark_verifier_sdk::snark_verifier::loader::evm::compile_solidity;

/// URL to download the Halo2 verifier's solidity code.
///
/// Note the following:
///
/// - OpenVM version is v1.4 (instead of v1.4.1) since patches don't include circuit-specific
///   changes.
///
/// - The commit points to [tag=v1.4][tag]
///
/// [tag]: https://github.com/openvm-org/openvm-solidity-sdk/releases/tag/v1.4
const URL_OPENVM_HALO2_VERIFIER: &str = "https://raw.githubusercontent.com/openvm-org/openvm-solidity-sdk/241058fb67da1439088f81633ea77b8c10015087/src/v1.4/Halo2Verifier.sol";

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

/// Download the published Halo2 verifier from GitHub and compile to binary.
pub(crate) fn download_and_compile() -> eyre::Result<Vec<u8>> {
    let sol_code: String = reqwest::blocking::Client::new()
        .get(URL_OPENVM_HALO2_VERIFIER)
        .send()?
        .error_for_status()?
        .text()?;

    Ok(compile_solidity(&sol_code))
}
