extern crate cargo;
extern crate glob;
extern crate semver;

use self::cargo::core::Package;
use self::cargo::util::Config;
use self::cargo::ops::{output_metadata, OutputMetadataOptions};
use self::glob::glob;
use std::path::Path;
use std::io::Write;

use common::{PACKAGE_TYPE, SourceUnit, SourceUnitMetadata};
use resolve::{resolve_dependencies, override_path_dependencies};

pub fn get_metadata(root: &Path, config: &Config, license: Option<String>) -> SourceUnitMetadata {
  let options = OutputMetadataOptions {
    features: vec![],
    manifest_path: &root,
    no_default_features: false,
    no_deps: false,
    version: 1,
  };

  match output_metadata(options, &config) {
    Ok(x) => {
      return SourceUnitMetadata::new(Some(x), license);
    },
    Err(_) => {
      println_stderr!("Could not get metadata for manifest path {}", root.to_str().unwrap());
      return SourceUnitMetadata::new(None, license);
    }
  }
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

fn construct_source_unit<'a>(manifest_file: &String, config: &'a Config) -> Result<SourceUnit, cargo::util::errors::CliError> {
  let root = Path::new(manifest_file);
  let package = try!(Package::for_path(&root, &config));
  let manifest = package.manifest();
  let dir = root.parent().unwrap().strip_prefix(config.cwd()).ok().unwrap();
  let metadata = get_metadata(&root, &config, manifest.metadata().license.clone());

  match override_path_dependencies(&package) {
    Some(package) => {
      Ok(SourceUnit::new(
        package.name().to_string(),
        PACKAGE_TYPE.to_string(),
        manifest.metadata().repository.clone(),
        get_files(&root.parent().unwrap(), &config),
        dir.to_str().unwrap().to_string(),
        resolve_dependencies(&package, &config),
        Some(metadata)
      ))
    },
    None => {
      Ok(SourceUnit::new(
        package.name().to_string(),
        PACKAGE_TYPE.to_string(),
        manifest.metadata().repository.clone(),
        get_files(&root.parent().unwrap(), &config),
        dir.to_str().unwrap().to_string(),
        resolve_dependencies(&package, &config),
        Some(metadata)
      ))
    }
  }
}

fn construct_source_units<'a>(manifest_files: &Vec<String>, config: &'a Config) -> Result<Vec<SourceUnit>, cargo::util::errors::CliError> {
  let mut source_units: Vec<SourceUnit> = Vec::<SourceUnit>::new();
  for manifest_file in manifest_files {
    source_units.push(construct_source_unit(manifest_file, &config).unwrap());
  }
  return Ok(source_units);
}

pub fn execute<'a>(config: &'a Config) -> Result<Vec<SourceUnit>, cargo::util::errors::CliError> {
  let manifest_files = find_all_manifest_files(config.cwd());
  return construct_source_units(&manifest_files, &config);
}
