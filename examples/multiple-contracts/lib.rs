#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod balance_checker {
    use ink::{
        env::{
            call::{build_call, ExecutionInput, Selector},
            DefaultEnvironment,
        },
        H160, U256,
    };

    #[ink(storage)]
    pub struct BalanceChecker {
        account: H160,
        token_contract: H160,
    }

    impl BalanceChecker {
        #[ink(constructor)]
        pub fn new(account: H160, token_contract: H160) -> Self {
            Self {
                account,
                token_contract,
            }
        }

        #[ink(message)]
        pub fn check(&self) -> u128 {
            build_call::<DefaultEnvironment>()
                .call(self.token_contract)
                .transferred_value(U256::zero())
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::balance_of")))
                        .push_arg(self.account),
                )
                .returns::<u128>()
                .invoke()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use drink::session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT};

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    #[drink::test]
    fn contracts_work_correctly(mut session: Session) -> Result<(), Box<dyn Error>> {
        let token_contract = session.deploy_bundle(
            BundleProvider::Psp22.bundle()?,
            "new",
            &["10", "None", "None", "1"],
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        let user_account = session.get_actor();
        let _checker_contract = session.deploy_bundle(
            BundleProvider::local()?,
            "new",
            &[user_account.to_string(), token_contract.to_string()],
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        let balance: u128 = session.call("check", NO_ARGS, NO_ENDOWMENT)??;
        assert_eq!(balance, 10);

        Ok(())
    }
}
