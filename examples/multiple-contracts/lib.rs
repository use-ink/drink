#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod checker {
    use ink::{
        env::{
            call::{build_call, ExecutionInput, Selector},
            DefaultEnvironment,
        },
        H160,
    };

    #[ink(storage)]
    pub struct Checker {
        contract: H160,
    }

    impl Checker {
        #[ink(constructor)]
        pub fn new(contract: H160) -> Self {
            Self { contract }
        }

        #[ink(message)]
        pub fn check(&self) -> bool {
            build_call::<DefaultEnvironment>()
                .call(self.contract)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get"
                ))))
                .returns::<bool>()
                .invoke()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use drink::session::{Session, NO_ARGS, NO_ENDOWMENT};

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    #[drink::test]
    fn contracts_work_correctly(mut session: Session) -> Result<(), Box<dyn Error>> {
        let contract = session.deploy_bundle(
            BundleProvider::Flipper.bundle()?,
            "new",
            &["true"],
            Some([1; 32]),
            NO_ENDOWMENT,
        )?;

        let _checker_contract = session.deploy_bundle(
            BundleProvider::local()?,
            "new",
            &[format!("{:?}", contract)],
            Some([2; 32]),
            NO_ENDOWMENT,
        )?;

        let value: bool = session.call("check", NO_ARGS, NO_ENDOWMENT)??;
        assert!(value);

        Ok(())
    }
}
