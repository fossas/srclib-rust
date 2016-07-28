extern crate cargo;
extern crate glob;
extern crate semver;

use self::cargo::core::{Package, Dependency};
use self::cargo::util::{Config};
use self::cargo::ops::{output_metadata, OutputMetadataOptions, ExportInfo, resolve_dependencies};
use self::glob::glob;
use std::collections::HashMap;
use std::path::Path;
use std::io::Write;

use common::{PACKAGE_TYPE, EncodableVersion, SourceUnit, ResolvedDependency};

pub fn get_metadata(root: &Path, config: &Config) -> Option<ExportInfo> {
  let options = OutputMetadataOptions {
    features: vec![],
    manifest_path: &root,
    no_default_features: false,
    no_deps: false,
    version: 1,
  };

  match output_metadata(options, &config) {
    Ok(x) => {
      return Some(x);
    },
    Err(_) => {
      println_stderr!("Could not get metadata for manifest path {}", root.to_str().unwrap());
      return None;
    }
  }
}

fn get_resolved_dependencies<'a>(package: &Package, config: &'a Config) -> Option<Vec<ResolvedDependency>> {
  let dependencies: Vec<Dependency> = package.dependencies().iter().map(|dep| dep.clone()).collect();
  let mut resolved_dependencies: Vec<ResolvedDependency> = Vec::new();
  let mut version_lookup: HashMap<String, semver::Version> = HashMap::new();

  match resolve_dependencies(&package, &config, None, vec![], false) {
    Ok((package_set, _)) => {
      for package_id in package_set.package_ids() {
        version_lookup.insert(package_id.name().to_string().clone(), package_id.version().clone());
      }
      for dependency in dependencies {
        match version_lookup.get(&dependency.name().to_string()).clone() {
          Some(version) => {
            resolved_dependencies.push(
              ResolvedDependency::new(
                &dependency,
                Some(EncodableVersion::new(&version))));
          },
          None => {
            resolved_dependencies.push(
              ResolvedDependency::new(&dependency, None)
            );
          }
        }
      }
    },
    Err(e) => {
      println_stderr!("Could not get dependencies: {:?}", e);
    }
  }

  Some(resolved_dependencies)
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

  Ok(SourceUnit::new(
    package.name().to_string(),
    PACKAGE_TYPE.to_string(),
    manifest.metadata().repository.clone(),
    get_files(&root.parent().unwrap(), &config),
    dir.to_str().unwrap().to_string(),
    get_resolved_dependencies(&package, &config),
    get_metadata(&root, &config)
  ))
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
