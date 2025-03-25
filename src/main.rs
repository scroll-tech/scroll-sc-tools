use clap::Parser;

mod commands;

mod digest;

mod verifier;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: commands::Commands,
}

fn main() -> eyre::Result<()> {
    let cmd = Cli::parse();

    cmd.commands.run()?;

    Ok(())
}
