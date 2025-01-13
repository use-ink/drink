//! Mocking utilities for contract calls.

mod contract;
mod error;
mod extension;
use std::collections::BTreeMap;

pub use contract::{mock_message, ContractMock, MessageMock, Selector};
use error::MockingError;
pub(crate) use extension::MockingExtension;
use ink_sandbox::pallet_revive::evm::H160;

/// Untyped result of a mocked call.
pub type MockedCallResult = Result<Vec<u8>, MockingError>;

/// A registry of mocked contracts.
pub(crate) struct MockRegistry {
    mocked_contracts: BTreeMap<H160, ContractMock>,
    nonce: u8,
}

impl MockRegistry {
    /// Creates a new registry.
    pub fn new() -> Self {
        Self {
            mocked_contracts: BTreeMap::new(),
            nonce: 0u8,
        }
    }

    /// Returns the salt for the next contract.
    pub fn salt(&mut self) -> [u8; 32] {
        self.nonce += 1;
        let mut salt = [0u8; 32];

        // Copy the bytes of `self.nonce` into the start of the `salt` array
        let nonce_size = std::mem::size_of_val(&self.nonce);
        salt[..nonce_size].copy_from_slice(&self.nonce.to_le_bytes());
        salt
    }

    /// Registers `mock` for `address`. Returns the previous mock, if any.
    pub fn register(&mut self, address: H160, mock: ContractMock) -> Option<ContractMock> {
        self.mocked_contracts.insert(address, mock)
    }

    /// Returns the mock for `address`, if any.
    pub fn get(&self, address: &H160) -> Option<&ContractMock> {
        self.mocked_contracts.get(address)
    }
}
