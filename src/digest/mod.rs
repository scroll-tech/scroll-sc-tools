use clap::Args;
use hex::ToHex;

mod compute;

#[derive(Debug, Args)]
pub struct ComputeCommand;

impl ComputeCommand {
    pub fn run(self) -> eyre::Result<()> {
        let (exe, leaf) = (
            scroll_zkvm_verifier::commitments::EXE_COMMIT,
            scroll_zkvm_verifier::commitments::VM_COMMIT,
        );

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

        println!("digest-1={}", digest_1.encode_hex::<String>());
        println!("digest-2={}", digest_2.encode_hex::<String>());

        Ok(())
    }
}
