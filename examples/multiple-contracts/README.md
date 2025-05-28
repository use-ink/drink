# Multiple contracts

You can easily work with multiple contracts at the same time, even if they are not part of the same project.

Both `#[drink::contract_bundle_provider]` and `#[drink::test]` macros take care of building all the contract crates that you declare in `Cargo.toml`.
Therefore, even if you are testing a huge suite of dapps, the only thing you have to do is to run
```rust
cargo test --release
```

## Scenario

We will use Flipper library as a dependency contract.
Simply declare it in `Cargo.toml`:
```toml
flipper = { path = "../flipper", default-features = false, features = [
    "ink-as-dependency",
] }
```

As usual, we have to include `ink-as-dependency` feature to use a contract as a dependency.

Locally, we have a contract that keeps two addresses:
 - a deployed Flipper contract's address
 - a user's address

The contract has a single message `check() -> bool`, which queries the Flipper contract for the current flipped value.
