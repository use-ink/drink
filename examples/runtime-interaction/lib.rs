#[cfg(test)]
mod tests {
    use drink::{
        minimal::{MinimalSandbox, RuntimeCall},
        pallet_balances, pallet_revive,
        sandbox_api::prelude::*,
        session::mocking_api::read_contract_binary,
        AccountId32, Sandbox,
    };

    #[test]
    fn we_can_make_a_token_transfer_call() {
        // We create a sandbox object, which represents a blockchain runtime.
        let mut sandbox = MinimalSandbox::default();

        // Bob will be the recipient of the transfer.
        const BOB: AccountId32 = AccountId32::new([2u8; 32]);

        // Firstly, let us check that the recipient account (`BOB`) is not the default actor, that
        // will be used as the caller.
        assert_ne!(MinimalSandbox::default_actor(), BOB);

        // Recipient's balance before the transfer.
        let initial_balance = sandbox.free_balance(&BOB);

        // Prepare a call object, a counterpart of a blockchain transaction.
        let call_object = RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
            dest: BOB.into(),
            value: 100,
        });

        // Submit the call to the runtime.
        sandbox
            .runtime_call(call_object, Some(MinimalSandbox::default_actor()))
            .expect("Failed to execute a call");

        // In the end, the recipient's balance should be increased by 100.
        assert_eq!(sandbox.free_balance(&BOB), initial_balance + 100);
    }

    #[test]
    fn we_can_work_with_the_contracts_pallet_in_low_level() {
        let mut sandbox = MinimalSandbox::default();

        // Construct the path to the contract file.
        let contract_path = std::path::Path::new(file!())
            .parent()
            .expect("Failed to determine the base path")
            .join("test-resources")
            .join("dummy.polkavm");

        // A few runtime calls are also available directly from the sandbox. This includes a part of
        // the contracts API.
        let actor = MinimalSandbox::default_actor();
        let origin = MinimalSandbox::convert_account_to_origin(actor);
        let upload_result = sandbox
            .upload_contract(read_contract_binary(&contract_path), origin, 1_000_000)
            .expect("Failed to upload a contract");

        // If a particular call is not available directly in the sandbox, it can always be executed
        // via the `runtime_call` method.
        let call_object = RuntimeCall::Revive(pallet_revive::Call::remove_code {
            code_hash: upload_result.code_hash,
        });

        sandbox
            .runtime_call(call_object, Some(MinimalSandbox::default_actor()))
            .expect("Failed to remove a contract");
    }
}
