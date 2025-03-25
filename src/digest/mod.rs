use clap::Args;

mod compute;

#[derive(Debug, Args)]
pub struct ComputeCommand {
    #[arg(long = "phase-1", help = "Compute digests for Euclid phase-1")]
    pub phase_1: bool,
    #[arg(long = "phase-2", help = "Compute digests for Euclid phase-2")]
    pub phase_2: bool,
}

impl ComputeCommand {
    pub fn run(self) -> eyre::Result<()> {
        if !(self.phase_1 ^ self.phase_2) {
            eyre::bail!("Please pass a single flag (--phase-1 or --phase-2)")
        }

        let (exe, leaf) = if self.phase_1 {
            (
                scroll_zkvm_verifier::commitments::bundle_euclidv1::EXE_COMMIT,
                scroll_zkvm_verifier::commitments::bundle_euclidv1::LEAF_COMMIT,
            )
        } else {
            (
                scroll_zkvm_verifier::commitments::bundle::EXE_COMMIT,
                scroll_zkvm_verifier::commitments::bundle::LEAF_COMMIT,
            )
        };

        let _digest_1 = compute::compress_commitment(&exe);
        let _digest_2 = compute::compress_commitment(&leaf);

        Ok(())
    }
}
