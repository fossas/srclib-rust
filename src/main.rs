use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use std::process;
use std::result::Result as StdResult;

use anyhow::{anyhow, Error, Result};
use cargo_metadata::{Dependency, Metadata, MetadataCommand, Package, PackageId};
use lazy_static::lazy_static;
use regex::Regex;
use semver::Version;
use structopt::StructOpt;

mod types;

use crate::types::*;

static MANIFEST_FILE: &str = "Cargo.toml";
static UNIT_TYPE: &str = "RustCargoPackage";
static USAGE: &str = "srclib-rust scan [--repo=<repo>] [--subdir=<subdir>]";

#[derive(Debug, StructOpt)]
struct ParsedArgs {
    #[structopt(long, default_value)]
    repo: String,
    #[structopt(long, default_value = ".")]
    pub subdir: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "srclib-rust", usage = USAGE)]
enum CmdRoot {
    Scan(ParsedArgs),
}

fn extract_pkg_version(id: &PackageId) -> Version {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d+\.\d+\.\d+").unwrap();
    }

    // Unwrapping is safe due to cargo guarantees
    RE.find(&id.repr).unwrap().as_str().parse().unwrap()
}

fn get_manifest_path<P: AsRef<Path>>(root: &P) -> PathBuf {
    root.as_ref().join(MANIFEST_FILE)
}

fn parse_args() -> ParsedArgs {
    match CmdRoot::from_args() {
        CmdRoot::Scan(parsed) => parsed,
    }
}

fn parse_cargo_meta(args: &ParsedArgs) -> Result<Metadata> {
    MetadataCommand::new()
        .manifest_path(get_manifest_path(&args.subdir)) // use subdir arg
        .exec()
        .map_err(|e| e.into())
}

fn xform_meta(meta: Metadata, args: &ParsedArgs) -> Result<Vec<SourceUnit>> {
    let self_packages = extract_self_packages(&meta);
    let reference_path = args.subdir.canonicalize()?;

    self_packages
        .iter()
        .map(|this_package| {
            // This is safe to unwrap/expect, since the manifest is a file
            // and all files have a parent directory.
            let package_dir = this_package.manifest_path.parent().expect("Unexpected manifest path");

            Ok(SourceUnitBuilder::default()
                .name(this_package.name.to_owned())
                .unit_type(UNIT_TYPE.to_owned())
                .repo(Some(args.repo.clone()))
                .files(get_source_files(package_dir, &reference_path.as_path())?)
                .deps(Some(get_direct_deps(&this_package, &meta, package_dir)?))
                .data(this_package.license.to_owned().into())
                .build()
                .map_err(|e| anyhow!(e))?)
        })
        .collect()
}

fn serialize_units(units: &[SourceUnit]) -> Result<String> {
    serde_json::to_string(&units).map_err(|e| e.into())
}

fn extract_self_packages(meta: &Metadata) -> Vec<&Package> {
    meta.packages
        .iter()
        .filter(|p| meta.workspace_members.contains(&p.id))
        .collect()
}

fn get_direct_deps<P: AsRef<Path>>(pkg: &Package, meta: &Metadata, root: P) -> Result<Vec<ResolvedDependency>> {
    let resolved = meta.resolve.as_ref().ok_or_else(|| anyhow!("Could not locate dependency information"))?;
    let root_node = resolved.nodes
        .iter()
        .find(|node| node.id == pkg.id)
        .ok_or_else(|| anyhow!(format!("Could not locate root node for package: {}", pkg.name)))?;        

    pkg.dependencies
        .iter()
        .cloned()
        .map(|dep| {
            let version_candidates: Vec<Version> = root_node.dependencies
            .iter()
            .filter(|id| id.repr.starts_with(&dep.name))
            .map(extract_pkg_version)
            .collect();
            try_dep_xform(dep, &root, &version_candidates)
        })
        .collect()
}

fn try_dep_xform<P: AsRef<Path>>(dep: Dependency, root: P, versions: &[Version]) -> Result<ResolvedDependency> {
    ResolvedDependencyBuilder::default()
        .name(dep.name.clone())
        .version(versions.iter().find(|version| dep.req.matches(version)).map(ToString::to_string))
        .optional(dep.optional)
        .source(
            dep.source
                .unwrap_or_else(|| "registry+https://github.com/rust-lang/crates.io-index".into()),
        )
        .scope(Some("normal".into()))
        .default_features(dep.uses_default_features)
        .features(dep.features)
        .cargo_toml_path(Some(get_manifest_path(&root).canonicalize()?.display().to_string()))
        .platform(dep.target.map(|p| p.to_string()))
        .build()
        .map_err(|e| anyhow!(e))
}

fn get_source_files<P: AsRef<Path>>(root: P, reference_path: P) -> Result<Vec<PathBuf>> {
    glob::glob(&root.as_ref().join("**/*.rs").display().to_string())?
        .map(|path| canonicalize_path(path, &reference_path))
        .collect()
}

fn canonicalize_path<E, P>(rp: StdResult<PathBuf, E>, reference: P) -> Result<PathBuf>
where
    E: StdError + Send + Sync + 'static,
    P: AsRef<Path>,
{
    rp.map(|p| {
        p.strip_prefix(reference.as_ref())
            // Unwrapping is safe because we only pick up files from within the root
            .unwrap()
            .to_owned()
    })
    .map_err(Into::<Error>::into)
}

fn ensure_index<P: AsRef<Path>>(root: P) -> Result<()> {
    let success = process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(root)
        .status()?
        .success();

    if !success {
        Err(anyhow!("An error occurred while generating the lockfile"))
    } else {
        Ok(())
    }
}

fn inner_main() -> Result<String> {
    let parsed = parse_args();
    // Using `cargo generate-lockfile`, we can update the cached index cheaply
    // without downloading whole crates.  This saves a lot of time and network
    // I/O. This takes a extra few seconds on cache hits, but can save tens of
    // minutes on misses.
    ensure_index(&parsed.subdir)?;
    let cargo = parse_cargo_meta(&parsed)?;
    let units = xform_meta(cargo, &parsed)?;
    serialize_units(&units)
}

fn main() {
    match inner_main() {
        Ok(output) => {
            println!("{}", output);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}
