use std::{env, fs, path::PathBuf};

use contract_transcode::ContractMessageTranscoder;
use drink::contract_api::ContractApi;
use sp_core::blake2_256;

use crate::app_state::{AppState, Contract};

pub fn build(app_state: &mut AppState) {
    let Ok(output) = std::process::Command::new("cargo")
        .arg("contract")
        .arg("build")
        .arg("--release")
        .output() else {
        app_state.print_error("Failed to execute build command. Make sure `cargo contract` is installed. (`cargo install cargo-contract`)");
        return;
    };

    if output.status.success() {
        app_state.print("Contract built successfully");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        app_state.print_error(&format!(
            "Failed to execute 'cargo contract' command:\n{stderr}"
        ));
    }
}

pub fn deploy(app_state: &mut AppState, constructor: String, salt: Vec<u8>) {
    // Get raw contract bytes
    let Some((contract_name, contract_file)) = find_wasm_blob() else {
        app_state.print_error("Failed to find contract file");
        return;
    };
    app_state.ui_state.contract_project_name = contract_name.clone();

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
        .pwd
        .join(format!("target/ink/{contract_name}.json"));

    let Ok(transcode) = ContractMessageTranscoder::load(metadata_path) else {
        app_state.print_error("Failed to create transcoder from metadata file.");
        return;
    };

    // Try deploying
    let result =
        app_state
            .sandbox
            .deploy_contract(contract_bytes, compute_selector(&constructor), salt);
    app_state.print_contract_action(&result);

    // Check if call has been executed successfully
    let result = match result.result {
        Ok(result) if result.result.did_revert() => {
            app_state.print_error(&format!(
                "Contract deployment failed with error: {:?}",
                result.result.data
            ));
            return;
        }
        Ok(result) => result,
        Err(err) => {
            app_state.print_error(&format!("Failed to deploy contract\n{err:?}"));
            return;
        }
    };

    // Everything went well
    app_state.chain_info.deployed_contracts += 1;
    app_state.contracts.push(Contract {
        name: contract_name,
        address: result.account_id,
        base_path: app_state.ui_state.pwd.clone(),
        transcode,
    });
    app_state.ui_state.current_contract = app_state.contracts.len() - 1;

    app_state.print("Contract deployed successfully");
}

pub fn call(app_state: &mut AppState, message: String) {
    let Some(account_id) = app_state.contracts.get(app_state.ui_state.current_contract)
        .map(|c| c.address.clone()) else {
        app_state.print_error("No deployed contract");
        return;
    };

    let result = app_state
        .sandbox
        .call_contract(account_id, compute_selector(&message));
    app_state.print_contract_action(&result);

    match result.result {
        Ok(result) if result.did_revert() => {
            app_state.print_error(&format!(
                "Contract call failed with error: {:?}",
                result.data
            ));
        }
        Ok(result) => {
            let result_decoded = match app_state
                .contracts
                .get(app_state.ui_state.current_contract)
                .unwrap()
                .transcode
                .decode_return(&message, &mut result.data.as_slice())
            {
                Ok(value) => value.to_string(),
                Err(err) => format!(
                    "Failed to decode return value: {err}. Raw bytes: {:?}",
                    result.data
                ),
            };
            app_state.print(&format!("Result: {:?}", result_decoded));
        }
        Err(err) => {
            app_state.print_error(&format!("Failed to deploy contract\n{err:?}"));
        }
    }
}

fn find_wasm_blob() -> Option<(String, PathBuf)> {
    let pwd = env::current_dir().expect("Failed to get current directory");
    let Ok(entries) = fs::read_dir(pwd.join("target/ink")) else {
        return None;
    };
    let Some(file) = entries
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().unwrap_or_default() == "wasm") else {
        return None;
    };

    let raw_name = file
        .file_name()
        .into_string()
        .expect("Invalid file name")
        .strip_suffix(".wasm")
        .expect("We have just checked file extension")
        .to_string();

    Some((raw_name, file.path()))
}

fn compute_selector(name: &str) -> Vec<u8> {
    let name = name.as_bytes();
    blake2_256(name)[..4].to_vec()
}
