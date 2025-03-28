# Scroll Security Council Tools

The repository offers tools for the Security Council to run and validate certain operations against Scroll's ZkVM [release](https://github.com/scroll-tech/zkvm-prover/releases/tag/v0.2.0)

## Setup

- Clone the `scroll-sc-tools` repository:
```shell
$ git clone git@github.com:scroll-tech/scroll-sc-tools.git && cd scroll-sc-tools
```
- [Install Rust](https://www.rust-lang.org/tools/install)
- Install Specific Nightly Toolchain (specified in [rust-toolchain.toml](./rust-toolchain.toml))
```
rustup toolchain install nightly-2025-02-14
```

## Generate Verifier

Scroll's ZkVM architecture proves Scroll's L2 blocks in layers (chunking -> batching -> bundling) where only the final layer (aka bundle) is an EVM-verifiable SNARK proof.

This proof is verified as part of the Bundle Finalization on-chain transaction.

The proof itself is verified by a `Verifier` contract, that's essentially a PLONK-verifier constructed using OpenVM's SDK. The `Verifier` contract is deployed on Sepolia and Mainnet.

The `generate-verifier` command allows one to trustlessly re-generate the verifier contract and prints out its codehash, that can be validated against on-chain available data.

### Prerequisite

In order to generate the verifier contract, we also need to first get the appropriate KZG trusted setup parameters. To get them from OpenVM's setup step, we first need to install
`openvm` and then go through the setup process.

* [Build and Install OpenVM](https://book.openvm.dev/getting-started/install.html#option-2-build-from-source) with [commit@3c35e9f](https://github.com/openvm-org/openvm/tree/3c35e9f369da8dee065a089fb72f3580af7dcaf9)
* Validate `openvm` version
```shell
$ cargo openvm --version

# should print
# cargo-openvm openvm (3c35e9f 2025-03-28T01:25:30.964028000Z)
```
* Setup KZG parameters (Note: requires ~200Gi memory and takes a long time to run)
```shell
$ cargo openvm setup
```

Upon finishing the above prerequisite, please go ahead and generate the verifier contract:
```shell
$ cargo run -- generate-verifier
```

## Compute Digests

The final layer (aka bundle) circuit is identified by two digests, namely `digest_1` and `digest_2`.

- `digest_1`: Attestation to the circuit code/logic. Any modification to the circuitry, to any layer including and below the final layer, will trigger a change to this digest value.
- `digest_2`: Attestation to the circuit config. The `openvm.toml` files configure each circuit (chunk/batch/bundle) and finally this digest value will change if any of those was modified.

An important requirement for "proof generation" to "on-chain verification" is that the on-chain verifier must populate these digests (constants) so as to disallow proof submitter to
potentially post digests for malicious circuitry. These digests are available on-chain in the deployed contracts on Sepolia and Mainnet.

An independent party can re-compute these digests from the ZkVM released circuitry and validate against on-chain values.

```shell
# Euclid Phase-1
$ cargo run -- compute-digest --phase-1

# Euclid Phase-2
$ cargo run -- compute-digest --phase-2
```
