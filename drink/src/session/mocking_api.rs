//! Mocking API for the sandbox.

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
    compile_module,
    pallet_revive::Config,
    session::mock::ContractMock, // DEFAULT_GAS_LIMIT,
};

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

        let origin = T::convert_account_to_origin(T::default_actor());
        let mock_address = self
            .sandbox()
            .deploy_contract(
                compile_module("dummy"),
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
            .register(mock_address.clone(), mock);

        mock_address
    }

    fn mock_existing_contract(&mut self, _mock: ContractMock, _address: H160) {
        todo!("soon")
    }
}
