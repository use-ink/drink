#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Fixed value returned by the example chain extension.
#[cfg(test)]
const CHAIN_EXTENSION_RETURN_VALUE: u32 = 100;

/// Here we put ink-side part of the example chain extension.
mod chain_extension_ink_side;

/// Here we put runtime-side part of the example chain extension.
#[cfg(test)]
mod chain_extension_runtime_side;

/// Simple ink! smart contract that calls a chain extension.
#[ink::contract(env = StakingEnvironment)]
mod contract_calling_chain_extension {
    use crate::chain_extension_ink_side::StakingEnvironment;

    #[ink(storage)]
    pub struct ContractCallingChainExtension {}

    impl ContractCallingChainExtension {
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn call_ce(&self) -> u32 {
            self.env().extension().get_num_of_validators()
        }
    }
}

#[cfg(test)]
mod tests {
    use drink::{
        create_sandbox,
        session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    };

    use crate::CHAIN_EXTENSION_RETURN_VALUE;

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    // We can inject arbitrary chain extension into the minimal runtime as follows:
    create_sandbox!(
        SandboxWithCE,
        crate::chain_extension_runtime_side::StakingExtension,
        drink::pallet_revive_debugging::DrinkDebug
    );

    /// Test that we can call chain extension from ink! contract and get a correct result.
    #[drink::test(sandbox = SandboxWithCE)]
    fn we_can_test_chain_extension(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
        let result: u32 = session
            .deploy_bundle_and(
                BundleProvider::local()?,
                "new",
                NO_ARGS,
                NO_SALT,
                NO_ENDOWMENT,
            )?
            .call("call_ce", NO_ARGS, NO_ENDOWMENT)??;

        assert_eq!(result, CHAIN_EXTENSION_RETURN_VALUE);

        Ok(())
    }
}
