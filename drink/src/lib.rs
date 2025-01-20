//! The drink crate provides a sandboxed runtime for testing smart contracts without a need for
//! a running node.

#![warn(missing_docs)]

pub mod errors;
pub mod pallet_revive_debugging;
#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "macros")]
pub use drink_test_macro::{contract_bundle_provider, test};
pub use errors::Error;
pub use frame_support;
pub use ink_sandbox::{
    api as sandbox_api, create_sandbox, pallet_balances, pallet_revive, pallet_timestamp,
    sp_externalities, AccountId32, DispatchError, Sandbox, Ss58Codec, Weight,
};
#[cfg(feature = "session")]
pub use session::mock::{mock_message, ContractMock, MessageMock, MockedCallResult, Selector};

/// Main result type for the drink crate.
pub type DrinkResult<T> = std::result::Result<T, Error>;

/// Minimal Sandbox runtime used for testing contracts with drink!.
#[allow(missing_docs)]
pub mod minimal {
    use ink_sandbox::create_sandbox;

    // create_sandbox!(MinimalSandbox);
    create_sandbox!(
        MinimalSandbox,
        (),
        crate::pallet_revive_debugging::DrinkDebug
    );
}

pub(crate) fn compile_module(contract_name: &str) -> Vec<u8> {
    use std::path::Path;
    // Get the current file's directory.
    let base_path = Path::new(file!())
        .parent()
        .and_then(Path::parent)
        .expect("Failed to determine the base path");

    // Construct the path to the contract file.
    let contract_path = base_path
        .join("test-resources")
        .join(format!("{}.polkavm", contract_name));

    std::fs::read(&contract_path).expect("Failed to read contract file")
}
