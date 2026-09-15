use std::io::Write;

use openvm_sdk::{
    Sdk,
    config::{AggregationSystemParams, AppConfig},
    prover::DeferralHookCommits,
};
use openvm_stark_sdk::config::{
    MAX_APP_LOG_STACKED_HEIGHT, app_params_with_100_bits_security,
    hook_params_with_100_bits_security,
};

/// The `openvm-solidity-sdk` release tag to download the verifier from.
///
/// This tracks the **solidity-sdk** release tag, which follows its own tagging
/// scheme and is *not* the same as the `openvm` crate version.
const SOLIDITY_SDK_TAG: &str = "v2.0";

/// The verifier variant published by the solidity SDK.
///
/// Scroll's bundle circuit defers proof verification, so it deploys the
/// deferral-enabled verifier. `v2.0-base` is for circuits that do not defer.
const VERIFIER_VARIANT: &str = "v2.0-deferral";

/// The flat Halo2 verifier emitted by `snark-verifier`.
const FILE_HALO2_VERIFIER: &str = "Halo2Verifier.sol";

/// The thin wrapper that Scroll deploys. It inherits `Halo2Verifier` and
/// exposes the friendlier `IOpenVmHalo2Verifier` interface.
const FILE_OPENVM_VERIFIER: &str = "OpenVmHalo2Verifier.sol";

/// Interface implemented by [`FILE_OPENVM_VERIFIER`].
const FILE_OPENVM_INTERFACE: &str = "interfaces/IOpenVmHalo2Verifier.sol";

/// Name of the contract whose bytecode is deployed on L1.
const CONTRACT_NAME: &str = "OpenVmHalo2Verifier";

/// Build the `solc` source unit name for a file in the verifier directory.
///
/// `solc` embeds a metadata hash in the compiled bytecode, and that hash covers
/// the source unit names as well as the compiler settings. These names must
/// therefore match the ones used by the OpenVM SDK verbatim, otherwise the
/// codehash differs from the deployed verifier's even though the executable
/// code is identical.
fn source_unit_name(file: &str) -> String {
    format!("src/{VERIFIER_VARIANT}/{file}")
}

/// Compile the three verifier sources and return the initcode of
/// [`CONTRACT_NAME`].
///
/// The settings mirror the OpenVM SDK exactly. `solc` enforces the compiler
/// version itself, since the sources pin `pragma solidity 0.8.19`.
fn compile_verifier(
    halo2_verifier_code: &str,
    openvm_verifier_code: &str,
    openvm_verifier_interface: &str,
) -> eyre::Result<Vec<u8>> {
    let solc_input = serde_json::json!({
        "language": "Solidity",
        "sources": {
            source_unit_name(FILE_HALO2_VERIFIER): { "content": halo2_verifier_code },
            source_unit_name(FILE_OPENVM_VERIFIER): { "content": openvm_verifier_code },
            source_unit_name(FILE_OPENVM_INTERFACE): { "content": openvm_verifier_interface },
        },
        "settings": {
            "remappings": ["forge-std/=lib/forge-std/src/"],
            "optimizer": {
                "enabled": true,
                "runs": 100_000,
                "details": { "constantOptimizer": false, "yul": false }
            },
            "evmVersion": "paris",
            "viaIR": false,
            "outputSelection": { "*": { "*": ["metadata", "evm.bytecode.object"] } }
        }
    });

    let mut child = std::process::Command::new("solc")
        .arg("--standard-json")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| eyre::eyre!("failed to spawn solc: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| eyre::eyre!("failed to open solc stdin"))?
        .write_all(solc_input.to_string().as_bytes())?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(eyre::eyre!("solc exited with status {}", output.status));
    }

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| eyre::eyre!("solc returned invalid JSON: {e}"))?;

    // `errors` also carries warnings, so only genuine errors are fatal.
    if let Some(errors) = parsed["errors"].as_array() {
        let fatal = errors
            .iter()
            .filter(|e| e["severity"] == "error")
            .map(|e| e["formattedMessage"].as_str().unwrap_or_default())
            .collect::<Vec<_>>();
        if !fatal.is_empty() {
            return Err(eyre::eyre!("solc reported errors:\n{}", fatal.join("\n")));
        }
    }

    let bytecode = parsed["contracts"][source_unit_name(FILE_OPENVM_VERIFIER)][CONTRACT_NAME]
        ["evm"]["bytecode"]["object"]
        .as_str()
        .ok_or_else(|| eyre::eyre!("could not find {CONTRACT_NAME} bytecode in solc output"))?
        .to_string();

    hex::decode(bytecode).map_err(|e| eyre::eyre!("solc returned invalid hex: {e}"))
}

/// Re-generate the EVM verifier from the circuit and return its initcode.
///
/// This mirrors `openvm-solidity-sdk`'s own `generate-verifier` tool, so the
/// emitted Solidity is already the canonical text and needs no reformatting
/// before it compiles to the published bytecode.
pub(crate) fn generate() -> eyre::Result<Vec<u8>> {
    let app_params = app_params_with_100_bits_security(MAX_APP_LOG_STACKED_HEIGHT);
    let agg_params = AggregationSystemParams::default();
    let hook_commits =
        DeferralHookCommits::from_system_params(&agg_params, hook_params_with_100_bits_security());

    let sdk = Sdk::builder()
        .app_config(AppConfig::riscv32(app_params))
        .agg_params(agg_params)
        .deferral_hook_commits(hook_commits)
        .build()?;

    let verifier = sdk.generate_halo2_verifier_solidity_with_version_name(VERIFIER_VARIANT)?;

    compile_verifier(
        &verifier.halo2_verifier_code,
        &verifier.openvm_verifier_code,
        &verifier.openvm_verifier_interface,
    )
}

/// Download the published verifier sources from GitHub and compile them.
pub(crate) fn download_and_compile() -> eyre::Result<Vec<u8>> {
    let client = reqwest::blocking::Client::new();
    let fetch = |file: &str| -> eyre::Result<String> {
        let url = format!(
            "https://raw.githubusercontent.com/openvm-org/openvm-solidity-sdk/{SOLIDITY_SDK_TAG}/src/{VERIFIER_VARIANT}/{file}"
        );
        Ok(client.get(&url).send()?.error_for_status()?.text()?)
    };

    compile_verifier(
        &fetch(FILE_HALO2_VERIFIER)?,
        &fetch(FILE_OPENVM_VERIFIER)?,
        &fetch(FILE_OPENVM_INTERFACE)?,
    )
}
