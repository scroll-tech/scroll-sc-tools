# Scroll Security Council Tools

The repository offers tools for the Security Council to run and validate certain operations against Scroll's ZkVM [release](https://github.com/scroll-tech/zkvm-prover/releases/tag/v0.8.0)

## Setup

- Clone the `scroll-sc-tools` repository:
```shell
$ git clone git@github.com:scroll-tech/scroll-sc-tools.git && cd scroll-sc-tools
```
- [Install Rust](https://www.rust-lang.org/tools/install)
- Install Specific Nightly Toolchain (specified in [rust-toolchain.toml](./rust-toolchain.toml))
```
rustup toolchain install nightly-2025-08-18
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

$ solc --version
```

* For `--recompute` only: install [Foundry](https://getfoundry.sh/) **v1.5.0**. The re-computed verifier source must be formatted with the same `forge fmt` version used by OpenVM to publish `Halo2Verifier.sol`, otherwise the embedded `solc` metadata hash (and thus the codehash) will not match the deployed verifier.

In order to generate the verifier contract, we can either download the source from [`OpenVM`](https://github.com/openvm-org/openvm-solidity-sdk/blob/v1.6/src/v1.6/Halo2Verifier.sol) or re-compute it.

* Download the verifier contract:
```shell
$ cargo run --release -- generate-verifier
```

* Re-compute the verifier contract (using [`OpenVM SDK`](https://github.com/openvm-org/openvm/blob/v1.6.0/crates/sdk/src/lib.rs#L805)):
```shell
# download SRS parameters
$ bash scripts/download-params.sh

# generate verifier
$ RUST_MIN_STACK=16777216 cargo run --release -- generate-verifier --recompute
```

Note: Re-computation requires very large amounts of computation and memory (~200 GB). It took about 4 minutes on AWS c7a.24xlarge.

Because `solc` embeds a metadata hash derived from the exact Solidity source text, the re-computed verifier must be formatted identically to the published `Halo2Verifier.sol` before compilation. `generate-verifier --recompute` therefore invokes `forge fmt` with the same formatter configuration used by OpenVM. For the resulting codehash to match the deployed verifier, the local `forge` binary should be **Foundry v1.5.0** (the version used to publish the OpenVM v1.6 verifier). Using a different `forge` version may produce a different codehash even though the verifier runtime logic is identical; in that case run `cargo run --release -- generate-verifier` (download mode) to obtain the exact on-chain codehash.

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
