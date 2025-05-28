# Quick start with `drink` library

This is a quick start guide introducing you to smart contract testing with `drink` library.
We will see how to write tests for a simple smart contract and make use of `drink`'s features.

## Prerequisites

You only need Rust installed (see [here](https://www.rust-lang.org/tools/install) for help).
Drink is developed and tested with stable Rust.

## Dependencies

You only need the `drink` library brought into your project:

```toml
drink = { version = "0.19.0-alpha" }
```

See [Cargo.toml](Cargo.toml) for a typical cargo setup of a single-contract project.

## Writing tests

### Preparing contracts

For every contract that you want to interact with from your tests, you need to create a _contract bundle_, which includes:

- built contract artifact (`.polkavm` file),
- contract transcoder (object based on the `.json` file, responsible for translating message arguments and results).

The recommended way is to use `drink::contract_bundle_provider` macro, which will discover all the contract dependencies (including the current crate, if that is the case) and gather all contract bundles into a single registry.

However, if needed, you can do it manually, by running `cargo contract build` for every such contract, and then, bring the artifacts into your tests.
For this, you might want to use `drink::session::ContractBundle` API, which includes `ContractBundle::load` and `local_contract_file!` utilities.

### `drink` test macros

`drink` provides a few macros to write tests for smart contracts:

- `#[drink::test]` - which marks a function as a test function (similar to `#[test]`).
- `#[drink::contract_bundle_provider]` - which gathers all contract artifacts into a single registry.

While neither is required to write `drink` tests, they make it easier to write and maintain them.

### Writing tests

Your typical test module will look like:

```rust
#[cfg(test)]
mod tests {
    use drink::session::{Session, NO_ARGS, NO_ENDOWMENT, NO_SALT};
    
    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    #[drink::test]
    fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn Error>> {
        let result: bool = session
            .deploy_bundle_and(BundleProvider::local(), "new", &["true"], NO_SALT, NO_ENDOWMENT)?
            .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
            .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
            .call_and("flip", NO_ARGS, NO_ENDOWMENT)?
            .call("get", NO_ARGS, NO_ENDOWMENT)??;
        assert_eq!(result, false);
    }
}
```

So, firstly, you declare a bundle provider like:

```rust
#[drink::contract_bundle_provider]
enum BundleProvider {}
```

It will take care of building all contract dependencies in the compilation phase and gather all contract bundles into a single registry.
Then, you will be able to get a contract bundle by calling:

```rust
let bundle = BundleProvider::local()?; // for the contract from the current crate
let bundle = BundleProvider::Flipper.bundle()?; // for the contract from the `flipper` crate
```

We mark each testcase with `#[drink::test]` attribute and declare return type as `Result` so that we can use the `?` operator:

```rust
#[drink::test]
fn testcase() -> Result<(), Box<dyn Error>> {
    // ...
}
```

Then, we can use the `Session` API to interact with both contracts and the whole runtime.
For details, check out testcases in [lib.rs](lib.rs).
