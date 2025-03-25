use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "")]
    GenerateVerifier(crate::verifier::GenerateCommand),
    #[command(about = "")]
    ComputeDigest(crate::digest::ComputeCommand),
}

impl Commands {
    pub fn run(self) -> eyre::Result<()> {
        match self {
            Commands::GenerateVerifier(cmd) => cmd.run(),
            Commands::ComputeDigest(cmd) => cmd.run(),
        }
    }
}
