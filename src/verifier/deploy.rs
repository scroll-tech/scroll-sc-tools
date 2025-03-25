use revm::{
    Context, Evm, Handler, InMemoryDB,
    primitives::{
        B256, Bytes, ExecutionResult, Output, TxEnv, TxKind, keccak256, specification::CancunSpec,
    },
};

/// Simulate deployment of initialisation code to get the deployed code and codehash.
pub(crate) fn deploy(init_code: &[u8]) -> eyre::Result<(Bytes, B256)> {
    let mut evm = Evm::new(
        Context::new_with_db(InMemoryDB::default()),
        Handler::mainnet::<CancunSpec>(),
    );

    *evm.tx_mut() = TxEnv {
        gas_limit: u64::MAX,
        transact_to: TxKind::Create,
        data: init_code.to_vec().into(),
        ..Default::default()
    };

    let result = evm.transact_commit()?;
    let code = match result {
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
