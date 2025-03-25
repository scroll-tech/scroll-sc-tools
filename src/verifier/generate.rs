use openvm_native_recursion::halo2::utils::{CacheHalo2ParamsReader, Halo2ParamsReader};
use openvm_sdk::{DefaultStaticVerifierPvHandler, Sdk, config::AggConfig};

/// The default directory to locate openvm's halo2 SRS parameters.
const DEFAULT_PARAMS_DIR: &str = concat!(env!("HOME"), "/.openvm/params/");

/// Generate and return the EVM PLONK verifier's initcode.
pub(crate) fn generate() -> eyre::Result<Vec<u8>> {
    let halo2_params_reader = CacheHalo2ParamsReader::new(DEFAULT_PARAMS_DIR);

    let agg_pk = Sdk.agg_keygen(
        AggConfig::default(),
        &halo2_params_reader,
        &DefaultStaticVerifierPvHandler,
    )?;

    let halo2_params =
        halo2_params_reader.read_params(agg_pk.halo2_pk.wrapper.pinning.metadata.config_params.k);

    Ok(snark_verifier_sdk::evm::gen_evm_verifier_shplonk::<
        snark_verifier_sdk::halo2::aggregation::AggregationCircuit,
    >(
        &halo2_params,
        agg_pk.halo2_pk.wrapper.pinning.pk.get_vk(),
        agg_pk.halo2_pk.wrapper.pinning.metadata.num_pvs.clone(),
        None,
    ))
}
