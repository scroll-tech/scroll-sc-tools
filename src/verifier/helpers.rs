use openvm_native_recursion::halo2::utils::Halo2ParamsReader;
use openvm_sdk::Sdk;
use snark_verifier_sdk::{
    SHPLONK,
    evm::gen_evm_verifier_sol_code,
    halo2::aggregation::AggregationCircuit,
    snark_verifier::loader::evm::compile_solidity,
};

/// URL to download the Halo2 verifier's solidity code.
///
/// Note the following:
///
/// - OpenVM version is v1.6 (instead of v1.6.x patches) since patches don't include
///   circuit-specific changes.
///
/// - The source points to [tag=v1.6][tag]
///
/// [tag]: https://github.com/openvm-org/openvm-solidity-sdk/releases/tag/v1.6
const URL_OPENVM_HALO2_VERIFIER: &str = "https://github.com/openvm-org/openvm-solidity-sdk/raw/refs/heads/main/src/v1.6/Halo2Verifier.sol";

/// Foundry formatter configuration used by OpenVM when publishing
/// `Halo2Verifier.sol`. `solc` embeds a metadata hash that depends on the exact
/// source text, so re-generated code must be formatted identically before
/// compilation in order to reproduce the on-chain codehash.
const FOUNDRY_FMT_TOML: &str = r#"[fmt]
sort_imports = true
bracket_spacing = true
int_types = "long"
line_length = 120
multiline_func_header = "attributes_first"
number_underscore = "thousands"
quote_style = "double"
single_line_statement_blocks = "single"
tab_width = 4
wrap_comments = false
"#;

/// Format Solidity source with `forge fmt` using the canonical formatter config.
fn format_solidity(sol_code: &str) -> eyre::Result<String> {
    let temp_dir = std::env::temp_dir().join("scroll-sc-tools-verifier-fmt");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir)?;

    let result = (|| {
        let sol_path = temp_dir.join("Halo2Verifier.sol");
        let config_path = temp_dir.join("foundry.toml");
        std::fs::write(&config_path, FOUNDRY_FMT_TOML)?;
        std::fs::write(&sol_path, sol_code)?;

        let format_output = std::process::Command::new("forge")
            .arg("fmt")
            .arg(&sol_path)
            .current_dir(&temp_dir)
            .output();

        match format_output {
            Ok(output) if output.status.success() => {
                std::fs::read_to_string(&sol_path).map_err(Into::into)
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(eyre::eyre!("forge fmt failed: {stderr}"))
            }
            Err(e) => Err(eyre::eyre!("failed to spawn forge fmt: {e}")),
        }
    })();

    let _ = std::fs::remove_dir_all(&temp_dir);
    result
}

/// Generate and return the EVM PLONK verifier's initcode.
pub(crate) fn generate() -> eyre::Result<Vec<u8>> {
    // The re-computed source must be formatted with the same Foundry version
    // used by OpenVM to publish Halo2Verifier.sol, otherwise the embedded
    // metadata hash (and thus the codehash) will differ.
    match std::process::Command::new("forge").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            if !version.contains("1.5.0") {
                eprintln!(
                    "Warning: local forge version is {}; for the re-computed codehash to match the deployed verifier, use Foundry v1.5.0.",
                    version.trim()
                );
            }
        }
        _ => eprintln!("Warning: could not determine local forge version."),
    }

    // Get the OpenVM SDK with the standard RISC-V 32-bit configuration. The Halo2
    // verifier only depends on the OpenVM aggregation circuit, which is the same
    // for all Scroll circuits that use this VM configuration.
    let sdk = Sdk::riscv32();
    let halo2_params_reader = sdk.halo2_params_reader();
    let halo2_pk = sdk.halo2_pk();
    let halo2_params = halo2_params_reader
        .read_params(halo2_pk.wrapper.pinning.metadata.config_params.k);

    // Generate the verifier Solidity code using the same method as
    // scroll-zkvm-prover's build-guest, then format it canonically before
    // compiling so the resulting codehash matches the deployed verifier.
    let sol_code = gen_evm_verifier_sol_code::<AggregationCircuit, SHPLONK>(
        &halo2_params,
        halo2_pk.wrapper.pinning.pk.get_vk(),
        halo2_pk.wrapper.pinning.metadata.num_pvs.clone(),
    );

    let sol_code = format_solidity(&sol_code)?;
    Ok(compile_solidity(&sol_code))
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
