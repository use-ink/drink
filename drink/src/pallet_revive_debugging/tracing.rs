use ink_sandbox::pallet_revive::evm::H160;
use parity_scale_codec::Encode;

use crate::{
    pallet_revive::{
        debug::{CallSpan, ExportedFunction},
        Config, ExecReturnValue, Tracing,
    },
    pallet_revive_debugging::DrinkDebug,
};

impl<R: Config> Tracing<R> for DrinkDebug {
    type CallSpan = DrinkCallSpan;

    fn new_call_span(
        contract_address: &H160,
        entry_point: ExportedFunction,
        input_data: &[u8],
    ) -> Self::CallSpan {
        DrinkCallSpan {
            contract_address: contract_address.clone(),
            entry_point,
            input_data: input_data.to_vec(),
        }
    }
}

/// A contract's call span.
///
/// It is created just before the call is made and `Self::after_call` is called after the call is
/// done.
pub struct DrinkCallSpan {
    /// The address of the contract that has been called.
    pub contract_address: H160,
    /// The entry point that has been called (either constructor or call).
    pub entry_point: ExportedFunction,
    /// The input data of the call.
    pub input_data: Vec<u8>,
}

impl CallSpan for DrinkCallSpan {
    fn after_call(self, output: &ExecReturnValue) {
        crate::pallet_revive_debugging::runtime::contract_call_debugger::after_call(
            self.contract_address.encode(),
            matches!(self.entry_point, ExportedFunction::Call),
            self.input_data.to_vec(),
            output.data.clone(),
        );
    }
}
