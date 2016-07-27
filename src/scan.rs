extern crate cargo;
extern crate glob;
extern crate rustc_serialize;

use cargo::core::{Package, Dependency};
use cargo::util::{Config};
use cargo::util::important_paths::{find_root_manifest_for_wd};
use glob::glob;
use rustc_serialize::json;

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

fn construct_source_unit() -> Result<SourceUnit, cargo::util::errors::CliError> {
    let config = Config::default().unwrap();
    let root = try!(find_root_manifest_for_wd(None, config.cwd()));
    let package = try!(Package::for_path(&root.as_path(), &config));
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

fn main() {
    let source_units = construct_source_unit().unwrap();
    let encoded = json::encode(&[source_units]).unwrap();
    println!("{}", encoded)
}
