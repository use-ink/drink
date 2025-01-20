use std::sync::Arc;

use contract_transcode::{ContractMessageTranscoder, Value};
use frame_system::Config as SysConfig;
use ink_sandbox::{
    pallet_revive::{self, evm::H160},
    ContractExecResultFor, ContractResultInstantiate, EventRecordOf,
};
use parity_scale_codec::{Decode, Encode};

use crate::{
    errors::MessageResult,
    minimal::{MinimalSandboxRuntime, RuntimeEvent},
    session::error::SessionError,
};

/// Data structure storing the results of contract interaction during a session.
///
/// # Naming convention
///
/// By `result` we mean the full result (enriched with some context information) of the contract
/// interaction, like `ContractExecResult`. By `return` we mean the return value of the contract
/// execution, like a value returned from a message or the address of a newly instantiated contract.
#[derive(frame_support::DefaultNoBound)]
pub struct Record<Config: pallet_revive::Config> {
    /// The results of contract instantiation.
    deploy_results: Vec<ContractResultInstantiate<Config>>,
    /// The return values of contract instantiation (i.e. the addresses of the newly instantiated
    /// contracts).
    deploy_returns: Vec<H160>,

    /// The results of contract calls.
    call_results: Vec<ContractExecResultFor<Config>>,
    /// The return values of contract calls (in the SCALE-encoded form).
    call_returns: Vec<Vec<u8>>,

    /// The events emitted by the contracts.
    event_batches: Vec<EventBatch<Config>>,
}

// API for `Session` to record results and events related to contract interaction.
impl<Config: pallet_revive::Config> Record<Config> {
    pub(super) fn push_deploy_result(&mut self, result: ContractResultInstantiate<Config>) {
        self.deploy_results.push(result);
    }

    pub(super) fn push_deploy_return(&mut self, return_value: H160) {
        self.deploy_returns.push(return_value);
    }

    pub(super) fn push_call_result(&mut self, result: ContractExecResultFor<Config>) {
        self.call_results.push(result);
    }

    pub(super) fn push_call_return(&mut self, return_value: Vec<u8>) {
        self.call_returns.push(return_value);
    }

    pub(super) fn push_event_batches(&mut self, events: Vec<EventRecordOf<Config>>) {
        self.event_batches.push(EventBatch { events });
    }
}

// API for the end user.
impl<Config: pallet_revive::Config> Record<Config> {
    /// Returns all the results of contract instantiations that happened during the session.
    pub fn deploy_results(&self) -> &[ContractResultInstantiate<Config>] {
        &self.deploy_results
    }

    /// Returns the last result of contract instantiation that happened during the session. Panics
    /// if there were no contract instantiations.
    pub fn last_deploy_result(&self) -> &ContractResultInstantiate<Config> {
        self.deploy_results.last().expect("No deploy results")
    }

    /// Returns all the return values of contract instantiations that happened during the session.
    pub fn deploy_returns(&self) -> &[H160] {
        &self.deploy_returns
    }

    /// Returns the last return value of contract instantiation that happened during the session.
    /// Panics if there were no contract instantiations.
    pub fn last_deploy_return(&self) -> &H160 {
        self.deploy_returns.last().expect("No deploy returns")
    }

    /// Returns all the results of contract calls that happened during the session.
    pub fn call_results(&self) -> &[ContractExecResultFor<Config>] {
        &self.call_results
    }

    /// Returns the last result of contract call that happened during the session. Panics if there
    /// were no contract calls.
    pub fn last_call_result(&self) -> &ContractExecResultFor<Config> {
        self.call_results.last().expect("No call results")
    }

    /// Returns all the (encoded) return values of contract calls that happened during the session.
    pub fn call_returns(&self) -> &[Vec<u8>] {
        &self.call_returns
    }

    /// Returns the last (encoded) return value of contract call that happened during the session.
    /// Panics if there were no contract calls.
    pub fn last_call_return(&self) -> &[u8] {
        self.call_returns.last().expect("No call returns")
    }

    /// Returns the last (decoded) return value of contract call that happened during the session.
    /// Panics if there were no contract calls.
    pub fn last_call_return_decoded<T: Decode>(&self) -> Result<MessageResult<T>, SessionError> {
        let mut raw = self.last_call_return();
        MessageResult::decode(&mut raw).map_err(|err| {
            SessionError::Decoding(format!(
                "Failed to decode the result of calling a contract: {err:?}"
            ))
        })
    }

    /// Returns all the event batches that were recorded for contract interactions during the
    /// session.
    pub fn event_batches(&self) -> &[EventBatch<Config>] {
        &self.event_batches
    }

    /// Returns the last event batch that was recorded for contract interactions during the session.
    /// Panics if there were no event batches.
    pub fn last_event_batch(&self) -> &EventBatch<Config> {
        self.event_batches.last().expect("No event batches")
    }
}

/// A batch of runtime events that were emitted during a single contract interaction.
pub struct EventBatch<R: SysConfig> {
    events: Vec<EventRecordOf<R>>,
}

impl<R: SysConfig> EventBatch<R> {
    /// Returns all the events that were emitted during the contract interaction.
    pub fn all_events(&self) -> &[EventRecordOf<R>] {
        &self.events
    }
}

impl EventBatch<MinimalSandboxRuntime> {
    /// Returns all the contract events that were emitted during the contract interaction.
    ///
    /// **WARNING**: This method will return all the events that were emitted by ANY contract. If your
    /// call triggered multiple contracts, you will have to filter the events yourself.
    ///
    /// We have to match against static enum variant, and thus (at least for now) we support only
    /// `MinimalSandbox`.
    pub fn contract_events(&self) -> Vec<&[u8]> {
        self.events
            .iter()
            .filter_map(|event| match &event.event {
                RuntimeEvent::Revive(
                    pallet_revive::Event::<MinimalSandboxRuntime>::ContractEmitted { data, .. },
                ) => Some(data.as_slice()),
                _ => None,
            })
            .collect()
    }

    /// The same as `contract_events`, but decodes the events using the given transcoder.
    ///
    /// **WARNING**: This method will try to decode all the events that were emitted by ANY
    /// contract. This means that some contract events might either fail to decode or be decoded
    /// incorrectly (to some rubbish). In the former case, they will be skipped, but with the latter
    /// case, you will have to filter the events yourself.
    ///
    /// **WARNING 2**: This method will ignore anonymous events.
    pub fn contract_events_decoded(
        &self,
        transcoder: &Arc<ContractMessageTranscoder>,
    ) -> Vec<Value> {
        let signature_topics = transcoder
            .metadata()
            .spec()
            .events()
            .iter()
            .filter_map(|event| event.signature_topic())
            .map(|sig| sig.as_bytes().try_into().unwrap())
            .collect::<Vec<[u8; 32]>>();

        self.contract_events()
            .into_iter()
            .filter_map(|data| {
                for signature_topic in &signature_topics {
                    if let Ok(decoded) = transcoder
                        // We have to `encode` the data because `decode_contract_event` is targeted
                        // at decoding the data from the runtime, and not directly from the contract
                        // events.
                        .decode_contract_event(&signature_topic, &mut &*data.encode())
                    {
                        return Some(decoded);
                    }
                }
                None
            })
            .collect()
    }
}
