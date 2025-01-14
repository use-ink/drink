#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// This is the classical flipper contract. It stores a single `bool` value in its storage. The
/// contract exposes:
/// - a constructor (`new`) that initializes the `bool` value to the given value,
/// - a message `flip` that flips the stored `bool` value from `true` to `false` or vice versa,
/// - a getter message `get` that returns the current `bool` value.
///
/// Additionally, we use the `debug_println` macro from the `ink_env` crate to produce some debug
/// logs from the contract.
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
            debug_println!("Initializing contract with: `{init}`");
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

/// We put `drink`-based tests as usual unit tests, into a test module.
#[cfg(test)]
mod tests {
    use drink::{
        sandbox_api::revive_api::decode_debug_buffer,
        session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT},
    };

    /// `drink` automatically discovers all the contract projects that your tests will need. For
    /// every such dependency (including the contract from the current crate), it will generate a
    /// [`ContractBundle`](drink::session::ContractBundle) object that contains the compiled contract's code
    /// and a special transcoder, which is used to encode and decode the contract's message
    /// arguments. Such a bundle will be useful when deploying a contract.
    ///
    /// To get a convenient way for obtaining such bundles, we can define an empty enum and mark
    /// it with the [`drink::contract_bundle_provider`](drink::contract_bundle_provider) attribute.
    /// From now on, we can use it in all testcases in this module.
    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    /// Now we write the simplest contract test, that will:
    /// 1. Deploy the contract.
    /// 2. Call its `flip` method.
    /// 3. Call its `get` method and ensure that the stored value has been flipped.
    ///
    /// We can use the [`drink::test`](drink::test) attribute to mark a function as a `drink` test.
    /// This way we ensure that all the required contracts are compiled and built, so that we don't
    /// have to run `cargo contract build` manually for every contract dependency.
    ///
    /// For convenience of using `?` operator, we mark the test function as returning a `Result`.
    ///
    /// `drink::test` will already provide us with a `Session` object. It is a wrapper around a runtime and it exposes
    /// a broad API for interacting with it. Session is generic over the runtime type, but usually and by default, we
    /// use `MinimalSandbox`, which is a minimalistic runtime that allows using smart contracts.
    #[drink::test]
    fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
        // Now we get the contract bundle from the `BundleProvider` enum. Since the current crate
        // comes with a contract, we can use the `local` method to get the bundle for it.
        let contract_bundle = BundleProvider::local()?;

        // We can now deploy the contract.
        let _contract_address = session.deploy_bundle(
            // The bundle that we want to deploy.
            contract_bundle,
            // The constructor that we want to call.
            "new",
            // The constructor arguments (as stringish objects).
            &["true"],
            // Salt for the contract address derivation.
            NO_SALT,
            // Initial endowment (the amount of tokens that we want to transfer to the contract).
            NO_ENDOWMENT,
        )?;

        // Once the contract is instantiated, we can call the `flip` method on the contract.
        session.call(
            // The message that we want to call.
            "flip",
            // The message arguments (as stringish objects). If none, then we can use the `NO_ARGS`
            // constant, which spares us from typing `&[]`.
            NO_ARGS,
            // Endowment (the amount of tokens that we want to transfer to the contract).
            NO_ENDOWMENT,
        )??;

        // Finally, we can call the `get` method on the contract and ensure that the value has been
        // flipped.
        //
        // `Session::call` returns a `Result<MessageResult<T>, SessionError>`, where `T` is the
        // type of the message result. In this case, the `get` message returns a `bool`, and we have
        // to explicitly hint the compiler about it.
        let result: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
        assert_eq!(result, false);

        Ok(())
    }

    /// In this testcase we will see how to get and read debug logs from the contract.
    #[drink::test]
    fn get_debug_logs(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
        session.deploy_bundle(
            BundleProvider::local()?,
            "new",
            &["true"],
            NO_SALT,
            NO_ENDOWMENT,
        )?;

        // `deploy_bundle` returns just a contract address. If we are interested in more details
        // about last operation (either deploy or call), we can get a `Record` object and use its
        // `last_deploy_result` (or analogously `last_call_result`) method, which will provide us
        // with a full report from the last contract interaction.
        //
        // In particular, we can get the decoded debug buffer from the contract. The buffer is
        // just a vector of bytes, which we can decode using the `decode_debug_buffer` function.
        let decoded_buffer = &session.record().last_deploy_result().debug_message;
        let encoded_buffer = decode_debug_buffer(decoded_buffer);

        assert_eq!(encoded_buffer, vec!["Initializing contract with: `true`"]);

        Ok(())
    }

    /// In this testcase we will see how to work with multiple contracts.
    #[drink::test]
    fn work_with_multiple_contracts(
        mut session: Session,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bundle = BundleProvider::local()?;

        // We can deploy the same contract multiple times. However, we have to ensure that the
        // derived contract addresses are different. We can do this by providing using different
        // arguments for the constructor or by providing a different salt.
        let first_address =
            session.deploy_bundle(bundle.clone(), "new", &["true"], NO_SALT, NO_ENDOWMENT)?;
        let _second_address =
            session.deploy_bundle(bundle.clone(), "new", &["true"], vec![0], NO_ENDOWMENT)?;
        let _third_address =
            session.deploy_bundle(bundle, "new", &["false"], NO_SALT, NO_ENDOWMENT)?;

        // By default, when we run `session.call`, `drink` will interact with the last deployed
        // contract.
        let value_at_third_contract: bool = session.call("get", NO_ARGS, NO_ENDOWMENT)??;
        assert_eq!(value_at_third_contract, false);

        // However, we can also call a specific contract by providing its address.
        let value_at_first_contract: bool =
            session.call_with_address(first_address, "get", NO_ARGS, NO_ENDOWMENT)??;
        assert_eq!(value_at_first_contract, true);

        Ok(())
    }
}
