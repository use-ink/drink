use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use contract_build::{BuildMode, ExecuteArgs, ManifestPath, OptimizationPasses, Verbosity};
use contract_transcode::ContractMessageTranscoder;

use crate::{
    app_state::{print::format_contract_action, AppState, Contract},
    executor::error::BuildError,
};

fn build_result(app_state: &mut AppState) -> Result<String, BuildError> {
    let path_to_cargo_toml = app_state.ui_state.cwd.join(Path::new("Cargo.toml"));
    let manifest_path = ManifestPath::new(path_to_cargo_toml.clone()).map_err(|err| {
        BuildError::InvalidManifest {
            manifest_path: path_to_cargo_toml,
            err,
        }
    })?;

    let args = ExecuteArgs {
        manifest_path,
        build_mode: BuildMode::Release,
        optimization_passes: Some(OptimizationPasses::default()),
        verbosity: Verbosity::Quiet,
        ..Default::default()
    };

    contract_build::execute(args)
        .map_err(|err| BuildError::BuildFailed { err })?
        .dest_wasm
        .ok_or(BuildError::WasmNotGenerated)?
        .canonicalize()
        .map_err(|err| BuildError::InvalidDestPath { err })
        .map(|pb| pb.to_string_lossy().to_string())
}

/// Build the contract in the current directory.
pub fn build(app_state: &mut AppState) {
    match build_result(app_state) {
        Ok(res) => app_state.print(&format!("Contract built successfully {res}")),
        Err(msg) => app_state.print_error(&format!("{msg}")),
    }
}

pub fn deploy(
    app_state: &mut AppState,
    constructor: String,
    args: Vec<String>,
    salt: Option<[u8; 32]>,
) {
    // Get raw contract bytes
    let Some((contract_name, contract_file)) = find_wasm_blob(&app_state.ui_state.cwd) else {
        app_state.print_error("Failed to find contract file");
        return;
    };

    let contract_bytes = match fs::read(contract_file) {
        Ok(bytes) => bytes,
        Err(err) => {
            app_state.print_error(&format!("Failed to read contract bytes\n{err}"));
            return;
        }
    };

    // Read contract metadata and prepare transcoder
    let metadata_path = app_state
        .ui_state
        .cwd
        .join(format!("target/ink/{contract_name}.json"));

    let Ok(transcoder) = ContractMessageTranscoder::load(metadata_path) else {
        app_state.print_error("Failed to create transcoder from metadata file.");
        return;
    };
    let transcoder = Arc::new(transcoder);

    match app_state.session.deploy(
        contract_bytes,
        &constructor,
        args.as_slice(),
        salt,
        None,
        &transcoder,
    ) {
        Ok(address) => {
            app_state.contracts.add(Contract {
                name: contract_name,
                address,
                base_path: app_state.ui_state.cwd.clone(),
                transcoder,
            });
            app_state.print("Contract deployed successfully");
        }
        Err(err) => app_state.print_error(&format!("Failed to deploy contract\n{err}")),
    }

    if let Some(info) = app_state.session.record().deploy_results().last() {
        app_state.print(&format_contract_action(info));
    }
}

pub fn call(app_state: &mut AppState, message: String, args: Vec<String>) {
    let Some(contract) = app_state.contracts.current_contract() else {
        app_state.print_error("No deployed contract");
        return;
    };

    let address = contract.address.clone();
    match app_state
        .session
        .call_with_address::<_, ()>(address, &message, &args, None)
    {
        Ok(result) => app_state.print(&format!("Result: {:?}", result)),
        Err(err) => app_state.print_error(&format!("Failed to call contract\n{err}")),
    };

    if let Some(info) = app_state.session.record().call_results().last() {
        app_state.print(&format_contract_action(info))
    }
}

fn find_wasm_blob(cwd: &Path) -> Option<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(cwd.join("target/ink")) else {
        return None;
    };
    let file = entries
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().unwrap_or_default() == "wasm")?;

    let raw_name = file
        .file_name()
        .into_string()
        .expect("Invalid file name")
        .strip_suffix(".wasm")
        .expect("We have just checked file extension")
        .to_string();

    Some((raw_name, file.path()))
}
