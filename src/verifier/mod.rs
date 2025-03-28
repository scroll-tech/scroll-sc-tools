use clap::Args;

mod deploy;

mod generate;

#[derive(Debug, Args)]
pub struct GenerateCommand;

impl GenerateCommand {
    pub fn run(self) -> eyre::Result<()> {
        let init_code = generate::generate()?;

        let (deployed_code, codehash) = deploy::deploy(&init_code)?;

        println!("verifier.bin code len={}", deployed_code.len());
        println!("verifier.bin codehash={:?}", codehash);

        Ok(())
    }
}
