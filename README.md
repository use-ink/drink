[![Rust checks](https://github.com/Cardinal-Cryptography/drink/actions/workflows/rust-checks.yml/badge.svg)](https://github.com/Cardinal-Cryptography/drink/actions/workflows/rust-checks.yml)
[![Built for ink!](https://raw.githubusercontent.com/paritytech/ink/master/.images/built-for-ink.svg)](https://github.com/paritytech/ink)

<h1 align="center"> DRink! </h1>
<p align="center"> <b>D</b>echained <b>R</b>eady-to-play <b>ink!</b> playground </p>

# What is DRink!?

## In brief

DRink! is a toolbox for ink! developers that allows for a fully functional ink! contract development without any running node.
It provides you with a unique, yet very powerful environment for interacting with contracts:
 - deploy and call your contracts synchronously, **without any delays** related to block production or networking
 - gain access to **powerful features** that are not available with standard methods like **contract mocking, enhanced debugging and call tracing**
 - work with **multiple contracts** at the same time
 - work with **arbitrary runtime** configurations, including custom chain extensions and runtime calls
 - have **full control over runtime state**, including block number, timestamp, etc.

## In detail

The key concept behind DRink! is to provide a nodeless environment.
To understand it fully, we need to have a high-level overview of the Substrate architecture.

_Note: While here we use Substrate-specific terms, these concepts are pretty universal and apply to at least most of the blockchain designs._

### 'Blockchain onion'

<img src="resources/blockchain-onion.svg">

Any blockchain network participant runs a single binary, which is usually called a _node_ or a _host_.
It is responsible for the fundamental operations like:
 - communication with other nodes (networking protocols, information dissemination, gossiping, etc.)
 - block production and finalization (consensus, block authoring, etc.)
 - storage (blockchain state, database, etc.)
 - sometimes also transaction pool, RPC, etc.

When it receives a new transaction (or a block), it has to update the blockchain state.
For that, it uses a _state transition function_, called a _runtime_.
This is an auxiliary binary, which serves as the core logic function, taking as an input the current state and a transaction, and returning the updated state.

In case the transaction is some smart contract interaction, the runtime has to execute it within an _isolated environment_.
(This is where the _contract pallet_ comes into play and spawns a dedicated sandbox.)

As a result, we have a layered architecture resembling an onion (actually, there are a few layers more, but we don't need to dig that deep).

### Testing strategies

Depending on the part of technology stack involved, we can derive three main testing strategies for smart contracts.

<img src="resources/testing-strategies.svg">


Before DRink!, you could have used ink!'s native test framework to execute either unit tests (with `#[ink::test]` macro) or end-to-end tests (with `#[ink_e2e::test]` macro).
DRink! enabled the third option, i.e. _quasi-end-to-end_ testing.

### quasi-E2E testing

This paradigm is a peculiar compromise between the two other strategies.
We give up the node layer (including networking, block production etc.), but we still have a fully functional runtime with attached storage.
In other words, we keep bare blockchain state in-memory, and we can interact with it directly however we want.

This way, we gain full control over the runtime, sacrificing real simulation of the blockchain environment.
However, usually, this is highly beneficial for the development process, as it allows for a much faster feedback loop, assisted with better insights into execution externalities.

---

# How to use DRink!?

You can use DRink! in three ways:

## Directly as a library

This way you gain access to full DRink! power in your test suites.
Check our helpful and verbose examples in the [examples](examples) directory.

`drink` library is continuously published to [crates.io](https://crates.io/crates/drink), so you can use it in your project with either `cargo add drink` or by adding the following line to your `Cargo.toml`:
```toml
drink = { version = "0.8" }
```

Full library documentation is available at: https://docs.rs/drink.

**Quick start guide** is available [here](examples/quick-start-with-drink/README.md).

## As an alternative backend to ink!'s E2E testing framework

DRink! is already integrated with ink! and can be used as a drop-in replacement for the standard E2E testing environment.
Just use corresponding argument in the test macro:
```rust
#[ink_e2e::test(backend = "runtime_only")]
```
to your test function and you have just switched from E2E testcase to quasi-E2E one, that doesn't use any running node in the background!

For a full example check out [ink! repository](https://github.com/paritytech/ink/blob/master/integration-tests/e2e-runtime-only-backend/lib.rs).

## With a command line tool

We provide a CLI which puts DRink! behind friendly TUI.
Below you can find a short demo of how it works.
For more details, consult [its README](drink-cli/README.md).

https://github.com/Cardinal-Cryptography/drink/assets/27450471/4a45ef8a-a7ec-4a2f-84ab-0a2a36c1cb4e

Similarly to `drink` library, `drink-cli` is published to [crates.io](https://crates.io/crates/drink-cli) as well.
You can install it with:
```shell
cargo install drink-cli
```
