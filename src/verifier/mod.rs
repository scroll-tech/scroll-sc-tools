use clap::Args;

mod deploy;

mod generate;

#[derive(Debug, Args)]
pub struct GenerateCommand;

impl GenerateCommand {
    pub fn run(self) -> eyre::Result<()> {
        let init_code = generate::generate()?;

        let (_deployed_code, _codehash) = deploy::deploy(&init_code)?;

        Ok(())
    }
}
