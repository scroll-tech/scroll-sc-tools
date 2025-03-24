mod generate;

mod utils;

fn main() -> eyre::Result<()> {
    // Generate the initialisation code for the EVM PLONK verifier.
    let init_code = generate::generate()?;

    // Get the deployed code and codehash of the EVM PLONK verifier contract.
    let (deployed_code, codehash) = utils::deploy(&init_code)?;

    println!(
        "verifier contract bytecode size = {:?}",
        deployed_code.len()
    );
    println!("verifier contract codehash      = {:?}", codehash);

    Ok(())
}
