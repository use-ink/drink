# Contract events

This example shows how we can extract events that were emitted by a contract.

When you are working with a `Session` object, you can consult its `Record` - a data structure that collects all the results and events that have been produced while interacting with contracts.
For example:

```rust
let mut session = Session::<MinimalSandbox>::default();
// .. some contract interaction

// `record` is a `Record` object that contains all the results and events that have been produced while interacting with contracts.
let record = session.record();
```

Given a `Record` object, we can extract the results of the contract interaction:

```rust
// `deploy_returns` returns a vector of contract addresses that have been deployed during the session.
let all_deployed_contracts = record.deploy_returns();
// `last_call_return_decoded` returns the decoded return value of the last contract call.
let last_call_value = record.last_call_return_decoded::<CustomType>();
```

as well as the events that have been emitted by contracts:

```rust
// `last_event_batch` returns the batch of runtime events that have been emitted during the last contract interaction.
let last_event_batch = record.last_event_batch();
// We can filter out raw events emitted by contracts with `contract_events` method.
let contract_events_data = last_event_batch.contract_events();
```
