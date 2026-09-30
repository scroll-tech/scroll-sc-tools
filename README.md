# Scroll Security Council Tools

The repository offers tools for the Security Council to run and validate certain operations against Scroll's ZkVM [release](https://github.com/scroll-tech/zkvm-prover/releases/tag/v0.9.0)

## Setup

- Clone the `scroll-sc-tools` repository:
```shell
$ git clone git@github.com:scroll-tech/scroll-sc-tools.git && cd scroll-sc-tools
```
- [Install Rust](https://www.rust-lang.org/tools/install)
- Install Specific Nightly Toolchain (specified in [rust-toolchain.toml](./rust-toolchain.toml))
```
rustup toolchain install nightly-2025-11-20
```

## Generate Verifier

Scroll's ZkVM architecture proves Scroll's L2 blocks in layers (chunking -> batching -> bundling) where only the final layer (aka bundle) is an EVM-verifiable SNARK proof.

This proof is verified as part of the Bundle Finalization on-chain transaction.

The proof itself is verified by a `Verifier` contract, that's essentially a PLONK-verifier constructed using OpenVM's SDK. The `Verifier` contract is deployed on Sepolia and Mainnet.

The `generate-verifier` command allows one to trustlessly re-generate the verifier contract and prints out its codehash, that can be validated against on-chain available data.

### Prerequisite

* Install solidity compiler via [solidity version manager](https://github.com/alloy-rs/svm-rs).
```shell
$ cargo install svm-rs

$ svm install 0.8.19

$ svm use 0.8.19

$ solc --version
```

* For `--recompute` only: install [Foundry](https://getfoundry.sh/) **v1.5.0**, the version OpenVM uses to format the published verifier.
```shell
$ foundryup --install v1.5.0

$ forge --version
```

The deployed contract is [`OpenVmHalo2Verifier`](https://github.com/openvm-org/openvm-solidity-sdk/blob/v2.0/src/v2.0-deferral/OpenVmHalo2Verifier.sol), a thin wrapper that inherits the flat [`Halo2Verifier`](https://github.com/openvm-org/openvm-solidity-sdk/blob/v2.0/src/v2.0-deferral/Halo2Verifier.sol). Scroll's bundle circuit defers proof verification, so it uses the `v2.0-deferral` variant rather than `v2.0-base`.

In order to generate the verifier contract, we can either download the source from [`OpenVM`](https://github.com/openvm-org/openvm-solidity-sdk/tree/v2.0/src/v2.0-deferral) or re-compute it.

* Download the verifier contract:
```shell
$ cargo run --release -- generate-verifier
```

* Re-compute the verifier contract (using [`OpenVM SDK`](https://github.com/openvm-org/openvm/tree/v2.0.0/crates/sdk)):
```shell
# download SRS parameters
$ bash scripts/download-params.sh

# generate verifier
$ RUST_MIN_STACK=16777216 cargo run --release -- generate-verifier --recompute
```

Note: Re-computation requires very large amounts of computation and memory (~200 GB). It took about 4 minutes on AWS c7a.24xlarge.

`solc` embeds a metadata hash that covers the source unit names and the compiler settings, and that hash is part of the deployed bytecode. Both modes therefore compile through `solc --standard-json` using the same source paths (`src/v2.0-deferral/...`) and settings as the OpenVM SDK: optimizer enabled with 100000 runs, `constantOptimizer` and the Yul optimizer both off, `evmVersion` `paris`, and the `forge-std/` remapping. Changing any of these changes the codehash even though the executable code is identical. The compiler version needs no separate check, since the sources pin `pragma solidity 0.8.19`.

The metadata hash also depends on the exact source text. The OpenVM SDK generates unformatted Solidity, while the published files are formatted with `forge fmt`. `--recompute` therefore formats the generated sources with OpenVM's `forge fmt` settings before compiling. Other Foundry versions may format differently and produce a different codehash.

For reference, release `v0.9.0` produces codehash `0x00dfb6855747412fa70b8a75aaa4950f1deafaace9d2a4e9923ad2e1a3589928`.

## Compute Digests

The final layer (aka bundle) circuit is identified by two digests, namely `digest_1` and `digest_2`.

- `digest_1`: Attestation to the circuit code/logic. Any modification to the circuitry, to any layer including and below the final layer, will trigger a change to this digest value.
- `digest_2`: Attestation to the circuit config. The `openvm.toml` files configure each circuit (chunk/batch/bundle) and finally this digest value will change if any of those was modified.

An important requirement for "proof generation" to "on-chain verification" is that the on-chain verifier must populate these digests (constants) so as to disallow proof submitter to
potentially post digests for malicious circuitry. These digests are available on-chain in the deployed contracts on Sepolia and Mainnet.

An independent party can re-compute these digests from the ZkVM released circuitry and validate against on-chain values.

```shell
$ cargo run --release -- compute-digest
```
