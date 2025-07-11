# Testing a chain extension

TODO: Replace with precompile example since chain extensions are deprecated in pallet-revive.

This example shows how you can use `drink` to test a chain extension.

From the perspective of a contract implementation or writing tests there is nothing special (i.e., more than usual) that you have to do to interact with a chain extension.
The thing that `drink` makes easier for you is combining an arbitrary chain extension with `drink`'s `MinimalSandbox`.
By simply calling:

```rust
create_sandbox!(
    SandboxWithCustomChainExtension,
    path::to::MyCustomChainExtension
);
```

you are provided with a `Sandbox` with a runtime that contains your custom chain extension and can be used to test your contract like:

```rust
Session::<SandboxWithCustomChainExtension>::default()
    .deploy_bundle_and(...)?
    .call(...)?
```
