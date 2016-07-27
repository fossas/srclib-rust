extern crate cargo;
extern crate glob;
extern crate rustc_serialize;

use cargo::core::{Package, Dependency};
use cargo::util::{Config};
// use cargo::util::important_paths::{find_root_manifest_for_wd};
use glob::glob;
use rustc_serialize::json;
use std::path::Path;

#[allow(dead_code, non_snake_case)]
#[derive(RustcEncodable)]
struct SourceUnit {
  Name: String,
  Type: String,
  Repo: Option<String>,
  Files: Vec<String>,
  Dir: String,
  Dependencies: Vec<Dependency>
}

fn find_all_manifest_files(root: &std::path::Path) -> Vec<String> {
  let mut v: Vec<String> = Vec::new();
  let glob_path_buf = root.join("**").join("Cargo.toml");

  for entry in glob(glob_path_buf.to_str().unwrap()).expect("Failed to read glob pattern") {
    match entry {
      Ok(path) => v.push(path.to_str().unwrap().to_string()),
      Err(e) => println!("{:?}", e),
    }
  }

  return v;
}

fn get_files(root: &std::path::Path, config: &Config) -> Vec<String> {
  let mut v: Vec<String> = Vec::new();
  let glob_path_buf = root.join("src").join("**").join("*.rs");

  for entry in glob(glob_path_buf.to_str().unwrap()).expect("Failed to read glob pattern") {
    match entry {
      Ok(path) => v.push(path.strip_prefix(config.cwd()).ok().unwrap().to_str().unwrap().to_string()),
      Err(e) => println!("{:?}", e),
    }
  }

  return v;
}

fn construct_source_unit(manifest_file: &String, config: &Config) -> Result<SourceUnit, cargo::util::errors::CliError> {
  // let root = try!(find_root_manifest_for_wd(None, config.cwd()));
  let root = Path::new(manifest_file);
  let package = try!(Package::for_path(&root, &config));
  let manifest = package.manifest();
  let dependencies : Vec<Dependency> = package.dependencies().iter().map(|dep| dep.clone()).collect();
  let dir = root.parent().unwrap().strip_prefix(config.cwd()).ok().unwrap();
  let files = get_files(&root.parent().unwrap(), &config);

  Ok(SourceUnit {
    Name: package.name().to_string(),
    Type: "RustCargoPackage".to_string(),
    Repo: manifest.metadata().repository.clone(),
    Files: files,
    Dir: dir.to_str().unwrap().to_string(),
    Dependencies: dependencies
  })
}

fn construct_source_units(manifest_files: &Vec<String>, config: &Config) -> Result<Vec<SourceUnit>, cargo::util::errors::CliError> {
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
  println!("{}", encoded)
}
