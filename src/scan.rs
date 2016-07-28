extern crate cargo;
extern crate glob;
extern crate rustc_serialize;
extern crate semver;

use cargo::core::{Package, Dependency};
use cargo::util::{Config};
use cargo::ops::{output_metadata, OutputMetadataOptions, ExportInfo, resolve_dependencies};
use glob::glob;
use rustc_serialize::{Encodable, Encoder, json};
use std::collections::HashMap;
use std::path::Path;
use std::io::Write;

macro_rules! println_stderr(
  ($($arg:tt)*) => { {
    let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
    r.expect("failed printing to stderr");
  } }
);

struct EncodableVersion {
  version: semver::Version
}

impl Encodable for EncodableVersion {
  fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
    format!("{}", self.version).encode(s)
  }
}

#[allow(dead_code, non_snake_case)]
#[derive(RustcEncodable)]
struct SourceUnitDependency {
  raw: Dependency,
  name: String,
  version: Option<EncodableVersion>
}

#[allow(dead_code, non_snake_case)]
#[derive(RustcEncodable)]
struct SourceUnit {
  Name: String,
  Type: String,
  Repo: Option<String>,
  Files: Vec<String>,
  Dir: String,
  Dependencies: Option<Vec<SourceUnitDependency>>,
  Metadata: Option<ExportInfo>
}

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

fn get_resolved_dependencies<'a>(package: &Package, config: &'a Config) -> Option<Vec<SourceUnitDependency>> {
  let dependencies: Vec<Dependency> = package.dependencies().iter().map(|dep| dep.clone()).collect();
  let mut resolved_dependencies: Vec<SourceUnitDependency> = Vec::new();
  let mut version_lookup: HashMap<String, semver::Version> = HashMap::new();

  match resolve_dependencies(&package, &config, None, vec![], false) {
    Ok((package_set, _)) => {
      for package_id in package_set.package_ids() {
        version_lookup.insert(package_id.name().to_string().clone(), package_id.version().clone());
      }
      for dependency in dependencies {
        match version_lookup.get(&dependency.name().to_string()).clone() {
          Some(version) => {
            resolved_dependencies.push(SourceUnitDependency {
              raw: dependency.clone(),
              name: dependency.name().to_string(),
              version: Some(EncodableVersion{
                version: version.clone()
              })
            });
          },
          None => {
            resolved_dependencies.push(SourceUnitDependency {
              raw: dependency.clone(),
              name: dependency.name().to_string(),
              version: None
            });
          }
        }
      }
    },
    Err(_) => {
      println_stderr!("Could not get dependencies");
    }
  }

  Some(resolved_dependencies)
}

fn find_all_manifest_files(root: &std::path::Path) -> Vec<String> {
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

fn get_files(root: &std::path::Path, config: &Config) -> Vec<String> {
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

  Ok(SourceUnit {
    Name: package.name().to_string(),
    Type: "RustCargoPackage".to_string(),
    Repo: manifest.metadata().repository.clone(),
    Files: get_files(&root.parent().unwrap(), &config),
    Dir: dir.to_str().unwrap().to_string(),
    Dependencies: get_resolved_dependencies(&package, &config),
    Metadata: get_metadata(&root, &config)
  })
}

fn construct_source_units<'a>(manifest_files: &Vec<String>, config: &'a Config) -> Result<Vec<SourceUnit>, cargo::util::errors::CliError> {
  let mut source_units: Vec<SourceUnit> = Vec::<SourceUnit>::new();
  for manifest_file in manifest_files {
    source_units.push(construct_source_unit(manifest_file, &config).unwrap());
  }
  return Ok(source_units);
}

fn main() {
  let config = Config::default().unwrap();
  let manifest_files = find_all_manifest_files(config.cwd());
  let source_units = construct_source_units(&manifest_files, &config).unwrap();
  let encoded = json::encode(&source_units).unwrap();
  println!("{}", encoded);
}
