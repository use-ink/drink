// TODO: No longer used due to pallet-revive removed `type Debug` for intercepting contract calls. Need to be fixed when we figure out the alternative approach. (#144)
//
// use std::sync::{Arc, Mutex};

// use ink_sandbox::pallet_revive::ExecReturnValue;
// use parity_scale_codec::{Decode, Encode};

// use crate::{
//     errors::MessageResult,
//     pallet_revive::chain_extension::ReturnFlags,
//     session::mock::{MockRegistry, Selector},
// };

// /// Runtime extension enabling contract call interception.
// pub(crate) struct MockingExtension {
//     /// Mock registry, shared with the sandbox.
//     ///
//     /// Potentially the runtime is executed in parallel and thus we need to wrap the registry in
//     /// `Arc<Mutex>` instead of `Rc<RefCell>`.
//     pub mock_registry: Arc<Mutex<MockRegistry>>,
// }

// impl MockingExtension {
//     fn intercept_call(
//         &self,
//         contract_address: Vec<u8>,
//         _is_call: bool,
//         input_data: Vec<u8>,
//     ) -> Vec<u8> {
//         let contract_address = Decode::decode(&mut &contract_address[..])
//             .expect("Contract address should be decodable");

//         match self
//             .mock_registry
//             .lock()
//             .expect("Should be able to acquire registry")
//             .get(&contract_address)
//         {
//             // There is no mock registered for this address, so we return `None` to indicate that
//             // the call should be executed normally.
//             None => None::<()>.encode(),
//             // We intercept the call and return the result of the mock.
//             Some(mock) => {
//                 let (selector, call_data) = input_data.split_at(4);
//                 let selector: Selector = selector
//                     .try_into()
//                     .expect("Input data should contain at least selector bytes");

//                 let result = mock
//                     .call(selector, call_data.to_vec())
//                     .expect("TODO: let the user define the fallback mechanism");

//                 // Although we don't know the exact type, thanks to the SCALE encoding we know
//                 // that `()` will always succeed (we only care about the `Ok`/`Err` distinction).
//                 let decoded_result: MessageResult<()> =
//                     Decode::decode(&mut &result[..]).expect("Mock result should be decodable");

//                 let flags = match decoded_result {
//                     Ok(_) => ReturnFlags::empty(),
//                     Err(_) => ReturnFlags::REVERT,
//                 };

//                 let result = ExecReturnValue {
//                     flags,
//                     data: result,
//                 };

//                 Some(result).encode()
//             }
//         }
//     }
// }
