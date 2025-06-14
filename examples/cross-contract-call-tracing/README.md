# Cross-contract call tracing

This example shows how you can trace and debug cross-contract calls.

## Scenario

Here we have a single contract with 3 methods:
 - `call_inner(arg: u32)`: returns the result of some simple computation on `arg`
 - `call_middle(next_callee: AccountId, arg: u32)`: calls `call_inner(arg)` at `next_callee` and forwards the result
 - `call_outer(next_callee: AccountId, next_next_callee: AccountId, arg: u32)`: calls `call_middle(next_next_callee, arg)` at `next_callee` and forwards the result

We deploy three instances of this contract, `inner`, `middle` and `outer`, and call `call_outer` on `outer` with `inner` and `middle` and some integer as arguments.

If we were using just `cargo-contract` or some other tooling, we would be able to see only the final result of the call.
However, it wouldn't be possible to trace the intermediate steps.
With `drink`, we can provide handlers for (synchronous) observing every level of the call stack.

## Running

```bash
cargo contract build --release
cargo test --release -- --show-output
```

You should be able to see similar output:
```
Contract at address `5CmHh6aBH6YZLjHGHjVtDDU4PfvDvk9s8n5xAcZQajxikksr` has been called with data: 
    new
and returned:
    ()

Contract at address `5FNvS4rLX8Y5NotoRzyBpmeNq2cfcSRpBWbHvgNrEiY3ero7` has been called with data: 
    new
and returned:
    ()

Contract at address `5DhNNsxhPMhg8R7StY3LbHraQWTDRFEbK2C1CaAD2AGvDCAf` has been called with data: 
    new
and returned:
    ()

Contract at address `5DhNNsxhPMhg8R7StY3LbHraQWTDRFEbK2C1CaAD2AGvDCAf` has been called with data: 
    inner_call { arg: 7 }
and returned:
    Ok(22)

Contract at address `5FNvS4rLX8Y5NotoRzyBpmeNq2cfcSRpBWbHvgNrEiY3ero7` has been called with data: 
    middle_call { next_callee: 5DhNNsxhPMhg8R7StY3LbHraQWTDRFEbK2C1CaAD2AGvDCAf, arg: 7 }
and returned:
    Ok(22)

Contract at address `5CmHh6aBH6YZLjHGHjVtDDU4PfvDvk9s8n5xAcZQajxikksr` has been called with data: 
    outer_call { next_callee: 5FNvS4rLX8Y5NotoRzyBpmeNq2cfcSRpBWbHvgNrEiY3ero7, next_next_callee: 5DhNNsxhPMhg8R7StY3LbHraQWTDRFEbK2C1CaAD2AGvDCAf, arg: 7 }
and returned:
    Ok(22)


successes:
    tests::test
```
