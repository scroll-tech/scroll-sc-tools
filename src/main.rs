use clap::Parser;

mod commands;

mod digest;

mod verifier;

#[derive(Parser)]
#[command(about = "Tools for Scroll's Security Council")]
struct Cli {
    #[command(subcommand)]
    commands: commands::Commands,
}

fn main() -> eyre::Result<()> {
    let cmd = Cli::parse();

    cmd.commands.run()?;

    Ok(())
}
