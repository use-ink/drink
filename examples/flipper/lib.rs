#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flipper {
    use ink::env::debug_println;

    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new(init: bool) -> Self {
            Self { value: init }
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            debug_println!("Previous value: `{}`", self.value);
            self.value = !self.value;
            debug_println!("Flipped to:     `{}`", self.value);
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            debug_println!("Reading value from storage");
            self.value
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
    fn initialization(mut session: Session) -> Result<(), Box<dyn Error>> {
        let contract = BundleProvider::local()?;
        let init_value: bool = session
            .deploy_bundle_and(contract, "new", &["true"], NO_SALT, NO_ENDOWMENT)?
            .call_and("get", NO_ARGS, NO_ENDOWMENT)?
            .record()
            .last_call_return_decoded()?
            .expect("Call was successful");

        assert_eq!(init_value, true);

        Ok(())
    }

    #[drink::test]
    fn flipping(mut session: Session) -> Result<(), Box<dyn Error>> {
        let contract = BundleProvider::Flipper.bundle()?;
        let init_value: bool = session
            .deploy_bundle_and(contract, "new", &["true"], NO_SALT, NO_ENDOWMENT)?
            .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
            .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
            .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
            .call_and("get", NO_ARGS, NO_ENDOWMENT)?
            .record()
            .last_call_return_decoded()?
            .expect("Call was successful");

        assert_eq!(init_value, false);

        Ok(())
    }
}
