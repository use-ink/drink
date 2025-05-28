//! Mocking API for the sandbox.

use std::path::Path;

use frame_support::sp_runtime::traits::Bounded;
use ink_primitives::DepositLimit;
use ink_sandbox::{
    api::prelude::*,
    pallet_revive::{
        evm::{H160, U256},
        MomentOf,
    },
    Sandbox, H256,
};

use super::{BalanceOf, Session};
use crate::{
    pallet_revive::Config,
    session::mock::ContractMock, // DEFAULT_GAS_LIMIT,
};

/// Read the contract binary file.
pub fn read_contract_binary(path: &std::path::PathBuf) -> Vec<u8> {
    std::fs::read(path).expect("Failed to read contract file")
}

/// Interface for basic mocking operations.
pub trait MockingApi<R: Config> {
    /// Deploy `mock` as a standard contract. Returns the address of the deployed contract.
    fn deploy(&mut self, mock: ContractMock) -> H160;

    /// Mock part of an existing contract. In particular, allows to override real behavior of
    /// deployed contract's messages.
    fn mock_existing_contract(&mut self, _mock: ContractMock, _address: H160);
}

impl<T: Sandbox> MockingApi<T::Runtime> for Session<T>
where
    T::Runtime: Config,
    BalanceOf<T::Runtime>: Into<U256> + TryFrom<U256> + Bounded,
    MomentOf<T::Runtime>: Into<U256>,
    <<T as Sandbox>::Runtime as frame_system::Config>::Hash: frame_support::traits::IsType<H256>,
{
    fn deploy(&mut self, mock: ContractMock) -> H160 {
        let salt = self
            .mocks
            .lock()
            .expect("Should be able to acquire lock on registry")
            .salt();

        // Construct the path to the contract file.
        let contract_path = Path::new(file!())
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("Failed to determine the base path")
            .join("test-resources")
            .join("dummy.polkavm");

        let origin = T::convert_account_to_origin(T::default_actor());
        let mock_address = self
            .sandbox()
            .deploy_contract(
                // Deploy a dummy contract to ensure that the pallet will treat the mock as a regular contract until it is
                // actually called.
                read_contract_binary(&contract_path),
                0u32.into(),
                vec![],
                Some(salt),
                origin,
                T::default_gas_limit(),
                DepositLimit::Unchecked,
            )
            .result
            .expect("Deployment of a dummy contract should succeed")
            .addr;

        self.mocks
            .lock()
            .expect("Should be able to acquire lock on registry")
            .register(mock_address, mock);

        mock_address
    }

    fn mock_existing_contract(&mut self, _mock: ContractMock, _address: H160) {
        todo!("soon")
    }
}
