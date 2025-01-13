use parity_scale_codec::Encode;
use sp_runtime_interface::runtime_interface;

use crate::sp_externalities::{decl_extension, ExternalitiesExt};

/// Contracts pallet outsources debug callbacks through this runtime interface.
///
/// Essentially, in our case, it just exposes extensions to the runtime.
///
/// At this level, data passed back/forth must be either primitive or implement some specific
/// traits. For simplicity, we just go with primitives and codec encoded data.
#[runtime_interface]
pub trait ContractCallDebugger {
    fn after_call(
        &mut self,
        contract_address: Vec<u8>,
        is_call: bool,
        input_data: Vec<u8>,
        result: Vec<u8>,
    ) {
        if let Some(ext) = self.extension::<TracingExt>() {
            ext.after_call(contract_address, is_call, input_data, result);
        }
    }

    fn intercept_call(
        &mut self,
        contract_address: Vec<u8>,
        is_call: bool,
        input_data: Vec<u8>,
    ) -> Option<Vec<u8>> {
        self.extension::<InterceptingExt>()
            .map(|ext| ext.intercept_call(contract_address, is_call, input_data))
    }
}

/// This trait describes a runtime extension that can be used to debug contract calls.
pub trait TracingExtT {
    /// Called after a contract call is made.
    fn after_call(
        &self,
        _contract_address: Vec<u8>,
        _is_call: bool,
        _input_data: Vec<u8>,
        _result: Vec<u8>,
    ) {
    }
}

decl_extension! {
    /// A wrapper type for the `TracingExtT` debug extension.
    pub struct TracingExt(Box<dyn TracingExtT + Send>);
}

/// This trait describes a runtime extension that can be used to intercept contract calls.
pub trait InterceptingExtT {
    /// Called when a contract call is made.
    ///
    /// The returned value must be a valid codec encoding for `Option<ExecResult>`.
    fn intercept_call(
        &self,
        _contract_address: Vec<u8>,
        _is_call: bool,
        _input_data: Vec<u8>,
    ) -> Vec<u8> {
        // By default, do not intercept, continue with the standard procedure.
        None::<()>.encode()
    }
}

decl_extension! {
    /// A wrapper type for the `InterceptingExtT` debug extension.
    pub struct InterceptingExt(Box<dyn InterceptingExtT + Send>);
}

/// The simplest extension - uses default implementation.
pub struct NoopExt;
impl TracingExtT for NoopExt {}
impl InterceptingExtT for NoopExt {}
