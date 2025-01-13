//! This module provides a context-aware interface for interacting with contracts.

use std::{
    fmt::Debug,
    mem,
    sync::{Arc, Mutex},
};

pub use contract_transcode;
use contract_transcode::ContractMessageTranscoder;
use error::SessionError;
use frame_support::{traits::fungible::Inspect, weights::Weight};
use ink_sandbox::{api::prelude::*, pallet_balances};
use ink_sandbox::{AccountIdFor, ContractExecResultFor, ContractInstantiateResultFor, Sandbox};
use parity_scale_codec::Decode;
pub use record::{EventBatch, Record};

use crate::{
    minimal::MinimalSandboxRuntime,
    pallet_revive::Config,
    pallet_revive_debugging::{InterceptingExt, TracingExt},
    session::mock::MockRegistry,
};

pub mod mock;
use mock::MockingExtension;
pub mod bundle;
pub mod error;
pub mod mocking_api;
mod record;
mod transcoding;

pub use bundle::ContractBundle;

use self::mocking_api::MockingApi;
use crate::{
    errors::MessageResult,
    // minimal::MinimalSandboxRuntime,
    session::transcoding::TranscoderRegistry,
};

type BalanceOf<R> = <<R as Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;
type HashFor<R> = <R as frame_system::Config>::Hash;

/// Convenient value for an empty sequence of call/instantiation arguments.
///
/// Without it, you would have to specify explicitly a compatible type, like:
/// `session.call::<String>(.., &[], ..)`.
pub const NO_ARGS: &[String] = &[];
/// Convenient value for an empty salt.
pub const NO_SALT: Vec<u8> = vec![];
/// Convenient value for no endowment.
///
/// Compatible with any runtime with `u128` as the balance type.
pub const NO_ENDOWMENT: Option<BalanceOf<MinimalSandboxRuntime>> = None;

/// Wrapper around `Sandbox` that provides a convenient API for interacting with multiple contracts.
///
/// Instead of talking with a low-level `Sandbox`, you can use this struct to keep context,
/// including: origin, gas_limit, transcoder and history of results.
///
/// `Session` has two APIs: chain-ish and for singular actions. The first one can be used like:
/// ```rust, no_run
/// # use std::sync::Arc;
/// # use contract_transcode::ContractMessageTranscoder;
/// # use ink_sandbox::AccountId32;
/// # use drink::{
/// #   session::Session,
/// #   session::{NO_ARGS, NO_SALT, NO_ENDOWMENT},
/// #   minimal::MinimalSandbox
/// # };
/// #
/// # fn get_transcoder() -> Arc<ContractMessageTranscoder> {
/// #   Arc::new(ContractMessageTranscoder::load("").unwrap())
/// # }
/// # fn contract_bytes() -> Vec<u8> { vec![] }
/// # fn bob() -> AccountId32 { AccountId32::new([0; 32]) }
///
/// # fn main() -> Result<(), drink::session::error::SessionError> {
///
/// Session::<MinimalSandbox>::default()
///     .deploy_and(contract_bytes(), "new", NO_ARGS, NO_SALT, NO_ENDOWMENT, &get_transcoder())?
///     .call_and("foo", NO_ARGS, NO_ENDOWMENT)?
///     .with_actor(bob())
///     .call_and("bar", NO_ARGS, NO_ENDOWMENT)?;
/// # Ok(()) }
/// ```
///
/// The second one serves for one-at-a-time actions:
/// ```rust, no_run
/// # use std::sync::Arc;
/// # use contract_transcode::ContractMessageTranscoder;
/// # use ink_sandbox::AccountId32;
/// # use drink::{
/// #   session::Session,
/// #   minimal::MinimalSandbox,
/// #   session::{NO_ARGS, NO_ENDOWMENT, NO_SALT}
/// # };
/// # fn get_transcoder() -> Arc<ContractMessageTranscoder> {
/// #   Arc::new(ContractMessageTranscoder::load("").unwrap())
/// # }
/// # fn contract_bytes() -> Vec<u8> { vec![] }
/// # fn bob() -> AccountId32 { AccountId32::new([0; 32]) }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
///
/// let mut session = Session::<MinimalSandbox>::default();
/// let _address = session.deploy(contract_bytes(), "new", NO_ARGS, NO_SALT, NO_ENDOWMENT, &get_transcoder())?;
/// let _result: u32 = session.call("foo", NO_ARGS, NO_ENDOWMENT)??;
/// session.set_actor(bob());
/// session.call::<_, ()>("bar", NO_ARGS, NO_ENDOWMENT)??;
/// # Ok(()) }
/// ```
///
/// You can also work with `.contract` bundles like so:
/// ```rust, no_run
/// # use drink::{
/// #   local_contract_file,
/// #   session::Session,
/// #   session::{ContractBundle, NO_ARGS, NO_SALT, NO_ENDOWMENT},
/// #   minimal::MinimalSandbox
/// # };
///
/// # fn main() -> Result<(), drink::session::error::SessionError> {
/// // Simplest way, loading a bundle from the project's directory:
/// Session::<MinimalSandbox>::default()
///     .deploy_bundle_and(local_contract_file!(), "new", NO_ARGS, NO_SALT, NO_ENDOWMENT)?; /* ... */
///
/// // Or choosing the file explicitly:
/// let contract = ContractBundle::load("path/to/your.contract")?;
/// Session::<MinimalSandbox>::default()
///     .deploy_bundle_and(contract, "new", NO_ARGS, NO_SALT, NO_ENDOWMENT)?; /* ... */
///  # Ok(()) }
/// ```
pub struct Session<T: Sandbox>
where
    T::Runtime: Config,
{
    sandbox: T,

    actor: AccountIdFor<T::Runtime>,
    gas_limit: Weight,

    transcoders: TranscoderRegistry<AccountIdFor<T::Runtime>>,
    record: Record<T::Runtime>,
    mocks: Arc<Mutex<MockRegistry<AccountIdFor<T::Runtime>>>>,
}

impl<T: Sandbox> Default for Session<T>
where
    T::Runtime: Config,
    T: Default,
{
    fn default() -> Self {
        let mocks = Arc::new(Mutex::new(MockRegistry::new()));
        let mut sandbox = T::default();

        sandbox.register_extension(InterceptingExt(Box::new(MockingExtension {
            mock_registry: Arc::clone(&mocks),
        })));

        Self {
            sandbox,
            mocks,
            actor: T::default_actor(),
            gas_limit: T::default_gas_limit(),
            transcoders: TranscoderRegistry::new(),
            record: Default::default(),
        }
    }
}

impl<T: Sandbox> Session<T>
where
    T::Runtime: Config + pallet_balances::Config,
{
    /// Sets a new actor and returns updated `self`.
    pub fn with_actor(self, actor: AccountIdFor<T::Runtime>) -> Self {
        Self { actor, ..self }
    }

    /// Returns currently set actor.
    pub fn get_actor(&self) -> AccountIdFor<T::Runtime> {
        self.actor.clone()
    }

    /// Sets a new actor and returns the old one.
    pub fn set_actor(&mut self, actor: AccountIdFor<T::Runtime>) -> AccountIdFor<T::Runtime> {
        mem::replace(&mut self.actor, actor)
    }

    /// Sets a new gas limit and returns updated `self`.
    pub fn with_gas_limit(self, gas_limit: Weight) -> Self {
        Self { gas_limit, ..self }
    }

    /// Sets a new gas limit and returns the old one.
    pub fn set_gas_limit(&mut self, gas_limit: Weight) -> Weight {
        mem::replace(&mut self.gas_limit, gas_limit)
    }

    /// Returns currently set gas limit.
    pub fn get_gas_limit(&self) -> Weight {
        self.gas_limit
    }

    /// Register a transcoder for a particular contract and returns updated `self`.
    pub fn with_transcoder(
        mut self,
        contract_address: AccountIdFor<T::Runtime>,
        transcoder: &Arc<ContractMessageTranscoder>,
    ) -> Self {
        self.set_transcoder(contract_address, transcoder);
        self
    }

    /// Registers a transcoder for a particular contract.
    pub fn set_transcoder(
        &mut self,
        contract_address: AccountIdFor<T::Runtime>,
        transcoder: &Arc<ContractMessageTranscoder>,
    ) {
        self.transcoders.register(contract_address, transcoder);
    }

    /// The underlying `Sandbox` instance.
    pub fn sandbox(&mut self) -> &mut T {
        &mut self.sandbox
    }

    /// Returns a reference to the record of the session.
    pub fn record(&self) -> &Record<T::Runtime> {
        &self.record
    }

    /// Returns a reference for mocking API.
    pub fn mocking_api(&mut self) -> &mut impl MockingApi<T::Runtime> {
        self
    }

    /// Deploys a contract with a given constructor, arguments, salt and endowment. In case of
    /// success, returns `self`.
    pub fn deploy_and<S: AsRef<str> + Debug>(
        mut self,
        contract_bytes: Vec<u8>,
        constructor: &str,
        args: &[S],
        salt: Vec<u8>,
        endowment: Option<BalanceOf<T::Runtime>>,
        transcoder: &Arc<ContractMessageTranscoder>,
    ) -> Result<Self, SessionError> {
        self.deploy(
            contract_bytes,
            constructor,
            args,
            salt,
            endowment,
            transcoder,
        )
        .map(|_| self)
    }
    fn record_events<V>(&mut self, recording: impl FnOnce(&mut Self) -> V) -> V {
        let start = self.sandbox.events().len();
        let result = recording(self);
        let events = self.sandbox.events()[start..].to_vec();
        self.record.push_event_batches(events);
        result
    }

    /// Deploys a contract with a given constructor, arguments, salt and endowment. In case of
    /// success, returns the address of the deployed contract.
    pub fn deploy<S: AsRef<str> + Debug>(
        &mut self,
        contract_bytes: Vec<u8>,
        constructor: &str,
        args: &[S],
        salt: Vec<u8>,
        endowment: Option<BalanceOf<T::Runtime>>,
        transcoder: &Arc<ContractMessageTranscoder>,
    ) -> Result<AccountIdFor<T::Runtime>, SessionError> {
        let data = transcoder
            .encode(constructor, args)
            .map_err(|err| SessionError::Encoding(err.to_string()))?;

        let result = self.record_events(|session| {
            session.sandbox.deploy_contract(
                contract_bytes,
                endowment.unwrap_or_default(),
                data,
                salt,
                session.actor.clone(),
                session.gas_limit,
                None,
            )
        });

        let ret = match &result.result {
            Ok(exec_result) if exec_result.result.did_revert() => {
                Err(SessionError::DeploymentReverted)
            }
            Ok(exec_result) => {
                let address = exec_result.account_id.clone();
                self.record.push_deploy_return(address.clone());
                self.transcoders.register(address.clone(), transcoder);

                Ok(address)
            }
            Err(err) => Err(SessionError::DeploymentFailed(*err)),
        };

        self.record.push_deploy_result(result);
        ret
    }

    /// Similar to `deploy` but takes the parsed contract file (`ContractBundle`) as a first argument.
    ///
    /// You can get it with `ContractBundle::load("some/path/your.contract")` or `local_contract_file!()`
    pub fn deploy_bundle<S: AsRef<str> + Debug>(
        &mut self,
        contract_file: ContractBundle,
        constructor: &str,
        args: &[S],
        salt: Vec<u8>,
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<AccountIdFor<T::Runtime>, SessionError> {
        self.deploy(
            contract_file.wasm,
            constructor,
            args,
            salt,
            endowment,
            &contract_file.transcoder,
        )
    }

    /// Performs a dry run of the deployment of a contract.
    pub fn dry_run_deployment<S: AsRef<str> + Debug>(
        &mut self,
        contract_file: ContractBundle,
        constructor: &str,
        args: &[S],
        salt: Vec<u8>,
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<ContractInstantiateResultFor<T::Runtime>, SessionError> {
        let data = contract_file
            .transcoder
            .encode(constructor, args)
            .map_err(|err| SessionError::Encoding(err.to_string()))?;

        Ok(self.sandbox.dry_run(|sandbox| {
            sandbox.deploy_contract(
                contract_file.wasm,
                endowment.unwrap_or_default(),
                data,
                salt,
                self.actor.clone(),
                self.gas_limit,
                None,
            )
        }))
    }

    /// Similar to `deploy_and` but takes the parsed contract file (`ContractBundle`) as a first argument.
    ///
    /// You can get it with `ContractBundle::load("some/path/your.contract")` or `local_contract_file!()`
    pub fn deploy_bundle_and<S: AsRef<str> + Debug>(
        mut self,
        contract_file: ContractBundle,
        constructor: &str,
        args: &[S],
        salt: Vec<u8>,
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<Self, SessionError> {
        self.deploy_bundle(contract_file, constructor, args, salt, endowment)
            .map(|_| self)
    }

    /// Uploads a raw contract code. In case of success, returns `self`.
    pub fn upload_and(mut self, contract_bytes: Vec<u8>) -> Result<Self, SessionError> {
        self.upload(contract_bytes).map(|_| self)
    }

    /// Uploads a raw contract code. In case of success returns the code hash.
    pub fn upload(&mut self, contract_bytes: Vec<u8>) -> Result<HashFor<T::Runtime>, SessionError> {
        let result = self
            .sandbox
            .upload_contract(contract_bytes, self.actor.clone(), None);

        result
            .map(|upload_result| upload_result.code_hash)
            .map_err(SessionError::UploadFailed)
    }

    /// Similar to `upload_and` but takes the contract bundle as the first argument.
    ///
    /// You can obtain it using `ContractBundle::load("some/path/your.contract")` or `local_contract_file!()`
    pub fn upload_bundle_and(self, contract_file: ContractBundle) -> Result<Self, SessionError> {
        self.upload_and(contract_file.wasm)
    }

    /// Similar to `upload` but takes the contract bundle as the first argument.
    ///
    /// You can obtain it using `ContractBundle::load("some/path/your.contract")` or `local_contract_file!()`
    pub fn upload_bundle(
        &mut self,
        contract_file: ContractBundle,
    ) -> Result<HashFor<T::Runtime>, SessionError> {
        self.upload(contract_file.wasm)
    }

    /// Calls a contract with a given address. In case of a successful call, returns `self`.
    pub fn call_and<S: AsRef<str> + Debug>(
        mut self,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<Self, SessionError> {
        // We ignore result, so we can pass `()` as the message result type, which will never fail
        // at decoding.
        self.call_internal::<_, ()>(None, message, args, endowment)
            .map(|_| self)
    }

    /// Calls the last deployed contract. In case of a successful call, returns `self`.
    pub fn call_with_address_and<S: AsRef<str> + Debug>(
        mut self,
        address: AccountIdFor<T::Runtime>,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<Self, SessionError> {
        // We ignore result, so we can pass `()` as the message result type, which will never fail
        // at decoding.
        self.call_internal::<_, ()>(Some(address), message, args, endowment)
            .map(|_| self)
    }

    /// Calls the last deployed contract. In case of a successful call, returns the encoded result.
    pub fn call<S: AsRef<str> + Debug, V: Decode>(
        &mut self,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<MessageResult<V>, SessionError> {
        self.call_internal::<_, V>(None, message, args, endowment)
    }

    /// Calls the last deployed contract. Expect it to be reverted and the message result to be of
    /// type `Result<_, E>`.
    pub fn call_and_expect_error<S: AsRef<str> + Debug, E: Debug + Decode>(
        &mut self,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<E, SessionError> {
        Ok(self
            .call_internal::<_, Result<(), E>>(None, message, args, endowment)
            .expect_err("Call should fail")
            .decode_revert::<Result<(), E>>()?
            .expect("Call should return an error")
            .expect_err("Call should return an error"))
    }

    /// Calls a contract with a given address. In case of a successful call, returns the encoded
    /// result.
    pub fn call_with_address<S: AsRef<str> + Debug, V: Decode>(
        &mut self,
        address: AccountIdFor<T::Runtime>,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<MessageResult<V>, SessionError> {
        self.call_internal(Some(address), message, args, endowment)
    }

    /// Performs a dry run of a contract call.
    pub fn dry_run_call<S: AsRef<str> + Debug>(
        &mut self,
        address: AccountIdFor<T::Runtime>,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<ContractExecResultFor<T::Runtime>, SessionError> {
        let data = self
            .transcoders
            .get(&address)
            .as_ref()
            .ok_or(SessionError::NoTranscoder)?
            .encode(message, args)
            .map_err(|err| SessionError::Encoding(err.to_string()))?;

        Ok(self.sandbox.dry_run(|sandbox| {
            sandbox.call_contract(
                address,
                endowment.unwrap_or_default(),
                data,
                self.actor.clone(),
                self.gas_limit,
                None,
            )
        }))
    }

    fn call_internal<S: AsRef<str> + Debug, V: Decode>(
        &mut self,
        address: Option<AccountIdFor<T::Runtime>>,
        message: &str,
        args: &[S],
        endowment: Option<BalanceOf<T::Runtime>>,
    ) -> Result<MessageResult<V>, SessionError> {
        let address = match address {
            Some(address) => address,
            None => self
                .record
                .deploy_returns()
                .last()
                .ok_or(SessionError::NoContract)?
                .clone(),
        };

        let data = self
            .transcoders
            .get(&address)
            .as_ref()
            .ok_or(SessionError::NoTranscoder)?
            .encode(message, args)
            .map_err(|err| SessionError::Encoding(err.to_string()))?;

        let result = self.record_events(|session| {
            session.sandbox.call_contract(
                address,
                endowment.unwrap_or_default(),
                data,
                session.actor.clone(),
                session.gas_limit,
                None,
            )
        });

        let ret = match &result.result {
            Ok(exec_result) if exec_result.did_revert() => {
                Err(SessionError::CallReverted(exec_result.data.clone()))
            }
            Ok(exec_result) => {
                self.record.push_call_return(exec_result.data.clone());
                self.record.last_call_return_decoded::<V>()
            }
            Err(err) => Err(SessionError::CallFailed(*err)),
        };

        self.record.push_call_result(result);
        ret
    }

    /// Set the tracing extension
    pub fn set_tracing_extension(&mut self, d: TracingExt) {
        self.sandbox.register_extension(d);
    }
}
