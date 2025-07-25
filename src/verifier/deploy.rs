use revm::{
    Context, ExecuteCommitEvm, MainBuilder, MainContext,
    context_interface::result::{ExecutionResult, Output},
    database::InMemoryDB,
    primitives::{B256, Bytes, TxKind, keccak256},
};

/// Simulate deployment of initialisation code to get the deployed code and codehash.
pub(crate) fn deploy(init_code: &[u8]) -> eyre::Result<(Bytes, B256)> {
    let bytecode: Bytes = init_code.to_vec().into();
    let ctx = Context::mainnet()
        .modify_tx_chained(|tx| {
            tx.kind = TxKind::Create;
            tx.data = bytecode;
        })
        .with_db(InMemoryDB::default());

    let mut evm = ctx.build_mainnet();

    let code = match evm.replay_commit()? {
        ExecutionResult::Success {
            output: Output::Create(code, _),
            ..
        } => code,
        ExecutionResult::Revert { gas_used, output } => {
            return Err(eyre::eyre!(
                "Contract deployment tx reverted: gas_used={gas_used}, output={:#x}",
                output
            ));
        }
        ExecutionResult::Halt { reason, gas_used } => {
            return Err(eyre::eyre!(
                "Contract deployment tx halted unexpectedly: gas_used={gas_used}, reason={:?}",
                reason
            ));
        }
        _ => unreachable!(),
    };

    let codehash = keccak256(&code);

    Ok((code, codehash))
}
