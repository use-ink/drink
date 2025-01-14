use std::{
    collections::{hash_map::Entry, HashMap},
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use cargo_metadata::{Metadata, MetadataCommand, Package};
use contract_build::{
    BuildArtifacts, BuildMode, ExecuteArgs, Features, ImageVariant, ManifestPath, Network,
    OptimizationPasses, OutputType, UnstableFlags, Verbosity, DEFAULT_MAX_MEMORY_PAGES,
};

use crate::bundle_provision::BundleProviderGenerator;

/// Contract package differentiator.
const INK_AS_DEPENDENCY_FEATURE: &str = "ink-as-dependency";

/// Stores the manifest paths of all contracts that have already been built.
///
/// This prevents from building the same contract for every testcase separately.
static CONTRACTS_BUILT: OnceLock<Mutex<HashMap<PathBuf, (String, PathBuf)>>> = OnceLock::new();

/// Build the current package with `cargo contract build --release` (if it is a contract package),
/// as well as all its contract dependencies. Return a collection of paths to corresponding
/// `.contract` files.
///
/// A package is considered as a contract package, if it has the `ink-as-dependency` feature.
///
/// A contract dependency, is a package defined in the `Cargo.toml` file with the
/// `ink-as-dependency` feature enabled.
pub fn build_contracts() -> BundleProviderGenerator {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("Error invoking `cargo metadata`");

    let (maybe_root, contract_deps) = get_contract_crates(&metadata);
    let maybe_root = maybe_root.map(build_contract_crate);
    let contract_deps = contract_deps.map(build_contract_crate);

    BundleProviderGenerator::new(
        maybe_root.clone().into_iter().chain(contract_deps),
        maybe_root.map(|(name, _)| name),
    )
}

/// Contract package together with the features it should be built with.
struct FeaturedPackage<'metadata> {
    package: &'metadata Package,
    features_on: Vec<String>,
}

fn get_contract_crates(
    metadata: &Metadata,
) -> (
    Option<FeaturedPackage>,
    impl Iterator<Item = FeaturedPackage>,
) {
    let pkg_lookup = |id| {
        metadata
            .packages
            .iter()
            .find(|package| package.id == id)
            .unwrap_or_else(|| panic!("Error resolving package {id}"))
    };

    let dep_graph = metadata
        .resolve
        .as_ref()
        .expect("Error resolving dependencies");

    let contract_deps = dep_graph
        .nodes
        .iter()
        .filter(|node| {
            node.features
                .contains(&INK_AS_DEPENDENCY_FEATURE.to_string())
        })
        .map(move |node| {
            let mut features_on = node.features.clone();
            features_on.retain(|feature| feature != INK_AS_DEPENDENCY_FEATURE && feature != "std");
            FeaturedPackage {
                package: pkg_lookup(node.id.clone()),
                features_on,
            }
        });

    let root = dep_graph
        .root
        .as_ref()
        .expect("Error resolving root package");
    let root = pkg_lookup(root.clone());

    (
        root.features
            .contains_key(INK_AS_DEPENDENCY_FEATURE)
            .then_some(FeaturedPackage {
                package: root,
                features_on: vec![],
            }),
        contract_deps,
    )
}

fn build_contract_crate(pkg: FeaturedPackage) -> (String, PathBuf) {
    let manifest_path = get_manifest_path(pkg.package);
    let mut features = Features::default();
    for feature in pkg.features_on {
        features.push(&feature);
    }

    match CONTRACTS_BUILT
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("Error locking mutex")
        .entry(manifest_path.clone().into())
    {
        Entry::Occupied(ready) => ready.get().clone(),
        Entry::Vacant(todo) => {
            let args = ExecuteArgs {
                manifest_path,
                verbosity: Verbosity::Default,
                build_mode: BuildMode::Release,
                features,
                network: Network::Online,
                build_artifact: BuildArtifacts::All,
                unstable_flags: UnstableFlags::default(),
                optimization_passes: Some(OptimizationPasses::default()),
                keep_debug_symbols: false,
                extra_lints: false,
                output_type: OutputType::HumanReadable,
                skip_wasm_validation: false,
                max_memory_pages: DEFAULT_MAX_MEMORY_PAGES,
                image: ImageVariant::Default,
            };

            let result = contract_build::execute(args).expect("Error building contract");
            let bundle_path = result
                .metadata_result
                .expect("Metadata should have been generated")
                .dest_bundle;

            let new_entry = (pkg.package.name.clone(), bundle_path);
            todo.insert(new_entry.clone());
            new_entry
        }
    }
}

fn get_manifest_path(package: &Package) -> ManifestPath {
    ManifestPath::new(package.manifest_path.clone().into_std_path_buf())
        .unwrap_or_else(|_| panic!("Error resolving manifest path for package {}", package.name))
}
