use clap::Args;

mod deploy;

mod helpers;

#[derive(Debug, Args)]
pub struct GenerateCommand {
    #[arg(
        long = "recompute",
        help = "Recompute the Halo2 Solidity Verifier",
        default_value = "false"
    )]
    pub recompute: bool,
}

impl GenerateCommand {
    pub fn run(self) -> eyre::Result<()> {
        let init_code = if self.recompute {
            helpers::generate()?
        } else {
            helpers::download_and_compile()?
        };

        let (deployed_code, codehash) = deploy::deploy(&init_code)?;

        println!("verifier.bin code len={}", deployed_code.len());
        println!("verifier.bin codehash={:?}", codehash);

        Ok(())
    }
}
