use std::path::{Path, PathBuf};
use std::process;

use anyhow::{anyhow, Result};
use cargo_metadata::{Dependency, Metadata, MetadataCommand, Package};
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
        .no_deps() // Don't get dependency meta
        .other_options(&["--offline".into()]) // don't make network calls
        .exec()
        .map_err(|e| e.into())
}

fn xform_meta(meta: Metadata, args: &ParsedArgs) -> Result<Vec<SourceUnit>> {
    let this_package: &Package = get_this_package(&meta);
    let src_unit = SourceUnitBuilder::default()
        .name(this_package.name.to_owned())
        .unit_type(UNIT_TYPE.to_owned())
        .repo(Some(args.repo.clone()))
        .files(get_source_files(&args.subdir)?)
        .deps(Some(get_direct_deps(&this_package, args)?))
        .data(this_package.license.to_owned().map(Into::<License>::into))
        .build()
        .map_err(|e| anyhow!(e))?;

    Ok(vec![src_unit])
}

fn serialize_units(units: &[SourceUnit]) -> Result<String> {
    serde_json::to_string(&units).map_err(|e| e.into())
}

fn get_this_package(meta: &Metadata) -> &Package {
    let source_id = meta
        .workspace_members
        .first()
        .expect("No workspace members found, cannot determine the correct package.");

    meta.packages
        .iter()
        .find(|p| p.id == *source_id)
        .expect("No known package matched the workspace member.")
}

fn get_direct_deps(pkg: &Package, args: &ParsedArgs) -> Result<Vec<ResolvedDependency>> {
    pkg.dependencies
        .iter()
        .cloned()
        .map(|dep| try_dep_xform(dep, args))
        .collect()
}

fn try_dep_xform(dep: Dependency, args: &ParsedArgs) -> Result<ResolvedDependency> {
    ResolvedDependencyBuilder::default()
        .name(dep.name)
        .version(Some(dep.req.to_string()))
        .optional(dep.optional)
        .source(
            dep.source
                .unwrap_or_else(|| "registry+https://github.com/rust-lang/crates.io-index".into()),
        )
        .scope(Some("normal".into()))
        .default_features(dep.uses_default_features)
        .features(dep.features)
        .cargo_toml_path(Some(get_manifest_path(&args.subdir).display().to_string()))
        .platform(dep.target.map(|p| p.to_string()))
        .build()
        .map_err(|e| anyhow!(e))
}

fn get_source_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>> {
    glob::glob(&root.as_ref().join("**/*.rs").display().to_string())?
        // Below is just an inverter between result and vec:
        // Vec<Result<PathBuf, GlobError>> -> Result<Vec<PathBuf>, anyhow::Error>
        .map(|r| r.map_err(|e| e.into()))
        .collect()
}

fn inner_main() -> Result<String> {
    let parsed = parse_args();
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
