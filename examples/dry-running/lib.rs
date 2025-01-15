#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod counter {
    #[ink(storage)]
    pub struct Counter {
        value: u32,
    }

    impl Counter {
        #[ink(constructor)]
        pub fn new(init: u32) -> Self {
            assert!(init < 10);
            Self { value: init }
        }

        #[ink(message)]
        pub fn increment(&mut self) {
            self.value = self.value.saturating_add(1);
        }

        #[ink(message)]
        pub fn get(&self) -> u32 {
            self.value
        }
    }
}

#[cfg(test)]
mod tests {
    use drink::{
        frame_support::sp_runtime::ModuleError,
        minimal::{MinimalSandbox, RuntimeCall},
        pallet_balances,
        sandbox_api::prelude::*,
        session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
        AccountId32, DispatchError, Sandbox,
    };

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    #[drink::test]
    fn we_can_dry_run_contract_interactions(
        mut session: Session,
    ) -> Result<(), Box<dyn std::error::Error>> {
        session.set_storage_deposit_limit(1_000_000);
        // Firstly, let us dry-run contract instantiation with an incorrect constructor argument.
        let result = session.dry_run_deployment(
            BundleProvider::local()?,
            "new",
            &["10"],
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        // Ensure that the contract was trapped.
        assert!(matches!(
            result.result,
            Err(DispatchError::Module(ModuleError{message: Some(error), ..})) if error == "ContractTrapped"
        ));
        // Ensure that no events were emitted.
        assert!(session.record().event_batches().is_empty());

        // Now, let deploy the contract with a correct constructor argument.
        let address = session.deploy_bundle(
            BundleProvider::local()?,
            "new",
            &["5"],
            NO_SALT,
            NO_ENDOWMENT,
        )?;
        // Ensure that deployment triggered event emission.
        assert!(!session.record().event_batches().is_empty());

        // Now, let us dry-run a contract call.
        let result = session.dry_run_call(address.clone(), "increment", NO_ARGS, NO_ENDOWMENT)?;
        // We can check the estimated gas consumption.
        let gas_estimation = result.gas_consumed;

        // In the end, we can execute the call and verify gas consumption.
        session.call_with_address::<_, ()>(address, "increment", NO_ARGS, NO_ENDOWMENT)??;
        let gas_consumption = session.record().last_call_result().gas_consumed;

        assert_eq!(gas_estimation, gas_consumption);

        Ok(())
    }

    #[test]
    fn we_can_dry_run_normal_runtime_transaction() {
        let mut sandbox = MinimalSandbox::default();

        // Bob will be the recipient of the transfer.
        let bob = AccountId32::new([2u8; 32]);

        // Recipient's balance before the transfer.
        let initial_balance = sandbox.free_balance(&bob);

        // Dry-run the transaction.
        sandbox.dry_run(|sandbox| {
            sandbox
                .runtime_call(
                    RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                        dest: bob.clone().into(),
                        value: 100,
                    }),
                    Some(MinimalSandbox::default_actor()),
                )
                .expect("Failed to execute a call")
        });

        // At the end, the balance of the recipient should remain unchanged and no events should have been emitted.
        assert_eq!(sandbox.free_balance(&bob), initial_balance);
        assert!(sandbox.events().is_empty());
    }
}
