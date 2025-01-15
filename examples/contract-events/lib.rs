#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod flipper {
    #[ink(event)]
    pub struct Flipped {
        new_value: bool,
    }

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
            self.value = !self.value;
            self.env().emit_event(Flipped {
                new_value: self.value,
            });
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
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
    fn we_can_inspect_emitted_events(mut session: Session) -> Result<(), Box<dyn Error>> {
        let bundle = BundleProvider::local()?;

        // Firstly, we deploy the contract and call its `flip` method.
        session.set_storage_deposit_limit(1_000_000);
        session.deploy_bundle(bundle.clone(), "new", &["false"], None, NO_ENDOWMENT)?;
        session.call("flip", NO_ARGS, NO_ENDOWMENT)??;

        // Now we can inspect the emitted events.
        let record = session.record();
        let contract_events = record
            .last_event_batch()
            // We can use the `contract_events_decoded` method to decode the events into
            // `contract_transcode::Value` objects.
            .contract_events_decoded(&bundle.transcoder);

        assert_eq!(contract_events.len(), 1);
        println!("flip_event: {:?}", &contract_events[0]);

        Ok(())
    }
}
