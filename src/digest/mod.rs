use clap::Args;
use hex::ToHex;

mod compute;

#[derive(Debug, Args)]
pub struct ComputeCommand;

/// Commitment copied from [`v0.7.0-rc.5`][bundle_exe_commit].
///
/// [bundle_exe_commit]: https://github.com/scroll-tech/zkvm-prover/blob/v0.7.0-rc.5/crates/circuits/bundle-circuit/bundle_exe_commit.rs
const EXE_COMMIT: [u32; 8] = [
    1144920578, 672155974, 440997509, 565686465, 150547879, 714324919, 1665603288, 907252136,
];

/// Commitment copied from [`v0.7.0-rc.5`][bundle_vm_commit].
///
/// [bundle_vm_commit]: https://github.com/scroll-tech/zkvm-prover/blob/v0.7.0-rc.5/crates/circuits/bundle-circuit/bundle_vm_commit.rs
const VM_COMMIT: [u32; 8] = [
    702922786, 974900043, 1870917533, 1628966797, 1650497578, 697799835, 298481193, 1937656708,
];

impl ComputeCommand {
    pub fn run(self) -> eyre::Result<()> {
        let (exe, leaf) = (EXE_COMMIT, VM_COMMIT);

        // Fr::to_bytes(&self) spits out little-endian bytes, so we reverse the order to finally
        // display the big-endian bytes in hex-encoded form.
        let digest_1 = compute::compress_commitment(&exe)
            .to_bytes()
            .into_iter()
            .rev()
            .collect::<Vec<u8>>();
        let digest_2 = compute::compress_commitment(&leaf)
            .to_bytes()
            .into_iter()
            .rev()
            .collect::<Vec<u8>>();

        println!("Feynman: digest-1={}", digest_1.encode_hex::<String>());
        println!("Feynman: digest-2={}", digest_2.encode_hex::<String>());

        Ok(())
    }
}
