extern crate cargo;
extern crate glob;
extern crate semver;

use self::cargo::core::{Package, Workspace};
use self::cargo::util::Config;
use self::cargo::util::important_paths::find_root_manifest_for_wd;
use self::cargo::ops::output_metadata;
use self::glob::glob;
use std::collections::BTreeSet;
use std::path::Path;
use std::io::Write;

use common::{PACKAGE_TYPE, get_output_metadata_options, SourceUnit, SourceUnitMetadata};
use resolve::{resolve_dependencies, override_path_dependencies};



pub fn get_metadata(ws: &Workspace, package: &Package, license: Option<String>) -> SourceUnitMetadata {
  match output_metadata(ws, &get_output_metadata_options()) {
    Ok(x) => {
      return SourceUnitMetadata::new(Some(x), license);
    },
    Err(_) => {
      println_stderr!("Could not get metadata for manifest path {}", package.manifest_path().to_str().unwrap());
      return SourceUnitMetadata::new(None, license);
    }
  }
}

fn get_files(root: &Path, config: &Config) -> Vec<String> {
  let mut v: Vec<String> = Vec::new();
  let glob_path_buf = root.join("**").join("*.rs");

  for entry in glob(glob_path_buf.to_str().unwrap()).expect("Failed to read glob pattern") {
    match entry {
      Ok(path) => v.push(path.strip_prefix(config.cwd()).ok().unwrap().to_str().unwrap().to_string()),
      Err(e) => println_stderr!("{:?}", e),
    }
  }

  return v;
}

fn construct_source_unit<'a>(ws: &Workspace, package: &Package) -> Result<SourceUnit, cargo::util::errors::CliError> {
  let manifest = package.manifest();
  let root = package.manifest_path().clone();
  let dir = root.parent().unwrap().strip_prefix(ws.config().cwd()).ok().unwrap();
  let metadata = get_metadata(ws, package, manifest.metadata().license.clone());

  println_stderr!("DEBUG: scanning: {:?}", root);

  match override_path_dependencies(&package) {
    Some(package) => {
      Ok(SourceUnit::new(
        package.name().to_string(),
        PACKAGE_TYPE.to_string(),
        manifest.metadata().repository.clone(),
        get_files(&root.parent().unwrap(), ws.config()),
        dir.to_str().unwrap().to_string(),
        resolve_dependencies(ws, &package),
        Some(metadata)
      ))
    },
    None => {
      Ok(SourceUnit::new(
        package.name().to_string(),
        PACKAGE_TYPE.to_string(),
        manifest.metadata().repository.clone(),
        get_files(&root.parent().unwrap(), ws.config()),
        dir.to_str().unwrap().to_string(),
        resolve_dependencies(ws, &package),
        Some(metadata)
      ))
    }
  }
}

fn construct_source_units<'a>(manifest_files: &Vec<String>, config: &'a Config) -> Result<Vec<SourceUnit>, cargo::util::errors::CliError> {
  let mut source_units: Vec<SourceUnit> = Vec::<SourceUnit>::new();
  let mut scanned_lookup: BTreeSet<String> = BTreeSet::new();
  for manifest_file in manifest_files {
    let manifest = try!(find_root_manifest_for_wd(Some(manifest_file.clone()), config.cwd()));
    let ws = try!(Workspace::new(&manifest, config));
    for package in ws.members() {
      if !scanned_lookup.contains(package.manifest_path().to_str().unwrap()) {
        scanned_lookup.insert(package.manifest_path().to_str().unwrap().to_string());
        source_units.push(construct_source_unit(&ws, &package).unwrap());
      }
    }
  }
  return Ok(source_units);
}

fn find_all_manifest_files(root: &Path) -> Vec<String> {
  let mut v: Vec<String> = Vec::new();
  let glob_path_buf = root.join("**").join("Cargo.toml");

  for entry in glob(glob_path_buf.to_str().unwrap()).expect("Failed to read glob pattern") {
    match entry {
      Ok(path) => v.push(path.to_str().unwrap().to_string()),
      Err(e) => println_stderr!("{:?}", e),
    }
  }

  return v;
}

pub fn execute<'a>(config: &'a Config) -> Result<Vec<SourceUnit>, cargo::util::errors::CliError> {
  let manifest_files = find_all_manifest_files(config.cwd());

  println_stderr!("DEBUG: found manifest files: {:?}", manifest_files);
  return construct_source_units(&manifest_files, &config);
}
