# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.18.0]

### Changed

- For ink! 5.1 support: Bump `ink`, `cargo-contract`, `frame-` and `sp-` crates.
- Use Rust 1.81 as the default toolchain.

## [0.17.0]

### Added

- Return error when call was reverted.

## [0.16.0]

### Changed

- Switch types from Rc to Arc.

## [0.15.0]

### Changed

- Migrate `Sandbox` related code to `ink_sandbox` crate.

## [0.14.0]

### Changed

- Bump `ink` to `5.0.0` and `cargo-contract` to `4.0.0`
- Rework Sandbox API to better support custom Runtime

## [0.13.0]

### Changed

- Bump `ink`, `cargo-contract`, `frame-` and `sp-` crates.

## [0.12.1]

### Added

- Support dry running contract interactions

## [0.12.0]

### Changed

- Hide macros behind dedicated `macros` (default) feature flag
- Hide contract bundles behind `session` feature flag

## [0.11.1]

### Added

- Respect features for the contract dependencies when building contracts via drink macros

## [0.11.0]

### Changed

- Support `ink@5.0.0-rc.2`
- Update `contract-*` crates to `4.0.0-rc.3`

### Changed

## [0.10.0]

### Changed

- Update toolchain to `1.74.0`
- Support `ink@5.0.0-rc.1`
- Update `contract-*` crates to `4.0.0-rc.2`

## [0.9.0]

### Changed

- Rework `Sandbox` API to ease working with custom runtimes

## [0.8.7]

### Changed

- Migrate examples back to `ink@4.3.0`
- Downgrade `contract-*` crates from `4.0.0-rc.1` to `3.2.0`
- Bumped toolchain to `1.74.0`

### Fixed

- Compilation issues due to the breaking changes in `contract-build` dependency

## [0.8.6] [YANKED]

### Added

- Accessing events emitted by contracts
- `#[drink::test]` creates and adds a `session: Session` argument to the test function

## [0.8.5] [YANKED]

### Changed

- Update `contract-*` crates from `3.x.x` to `4.0.0-rc.1`
- Migrate examples from `ink@4.2.1` to `ink@5.0.0-rc`

## [0.8.4]

### Added

- `NO_SALT`, `NO_ENDOWMENT` constants added
