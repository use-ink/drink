use drink::pallet_revive::chain_extension::{
    ChainExtension, Config as ContractsConfig, Environment, Ext, InitState, RetVal,
};
use scale::Encode;

use crate::CHAIN_EXTENSION_RETURN_VALUE;

/// Simple chain extension that provides some mocked data.
#[derive(Default)]
pub struct StakingExtension;

impl<Runtime: ContractsConfig> ChainExtension<Runtime> for StakingExtension {
    fn call<E: Ext<T = Runtime>>(
        &mut self,
        env: Environment<E, InitState>,
    ) -> drink::pallet_revive::chain_extension::Result<RetVal> {
        // Ensure that the contract called extension method with id `41`.
        assert_eq!(env.func_id(), 41);

        // Write fixed result of the extension call into the return buffer.
        env.buf_in_buf_out()
            .write(&CHAIN_EXTENSION_RETURN_VALUE.encode(), false, None)
            .expect("Failed to write result");

        // Return `Converging(0)` to indicate that the extension call has finished successfully.
        Ok(RetVal::Converging(0))
    }
}
