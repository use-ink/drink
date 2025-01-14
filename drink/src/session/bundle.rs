//! This module provides simple utilities for loading and parsing `.contract` files in context of `drink` tests.

use std::{path::PathBuf, sync::Arc};

use contract_metadata::ContractMetadata;
use contract_transcode::ContractMessageTranscoder;

use crate::{DrinkResult, Error};

/// A struct representing the result of parsing a `.contract` bundle file.
///
/// It can be used with the following methods of the `Session` struct:
/// - `deploy_bundle`
/// - `deploy_bundle_and`
/// - `upload_bundle`
/// - `upload_bundle_and`
#[derive(Clone)]
pub struct ContractBundle {
    /// Contract code bytes
    pub code: Vec<u8>,
    /// Transcoder derived from the ABI/metadata
    pub transcoder: Arc<ContractMessageTranscoder>,
}

impl ContractBundle {
    /// Load and parse the information in a `.contract` bundle under `path`, producing a
    /// `ContractBundle` struct.
    pub fn load<P>(path: P) -> DrinkResult<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let metadata: ContractMetadata = ContractMetadata::load(&path).map_err(|e| {
            Error::BundleLoadFailed(format!("Failed to load the contract file:\n{e:?}"))
        })?;

        let ink_metadata = serde_json::from_value(serde_json::Value::Object(metadata.abi))
            .map_err(|e| {
                Error::BundleLoadFailed(format!(
                    "Failed to parse metadata from the contract file:\n{e:?}"
                ))
            })?;

        let transcoder = Arc::new(ContractMessageTranscoder::new(ink_metadata));

        let code = metadata
            .source
            .wasm
            .ok_or(Error::BundleLoadFailed(
                "Failed to get the WASM blob from the contract file".to_string(),
            ))?
            .0;

        Ok(Self { code, transcoder })
    }

    /// Load the `.contract` bundle (`contract_file_name`) located in the `project_dir`` working directory.
    ///
    /// This is meant to be used predominantly by the `local_contract_file!` macro.
    pub fn local(project_dir: &str, contract_file_name: String) -> Self {
        let mut path = PathBuf::from(project_dir);
        path.push("target");
        path.push("ink");
        path.push(contract_file_name);
        Self::load(path).expect("Loading the local bundle failed")
    }
}

/// A convenience macro that allows you to load a bundle found in the target directory
/// of the current project.
#[macro_export]
macro_rules! local_contract_file {
    () => {
        drink::session::ContractBundle::local(
            env!("CARGO_MANIFEST_DIR"),
            env!("CARGO_CRATE_NAME").to_owned() + ".contract",
        )
    };
}
