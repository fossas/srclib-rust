extern crate cargo;
extern crate glob;
extern crate semver;

use self::cargo::core::{Dependency, Manifest, Package, SourceId, Summary, Workspace};
use self::cargo::sources::CRATES_IO;
use self::cargo::ops;
use std::collections::HashMap;
use std::path::Path;
use std::io::Write;

use common::{get_output_metadata_options, EncodableVersion, ResolvedDependency};

pub fn resolve_dependencies<'a>(ws: &Workspace, package: &Package) -> Option<Vec<ResolvedDependency>> {
  let dependencies: Vec<Dependency> = package.dependencies().iter().map(|dep| dep.clone()).collect();
  let mut resolved_dependencies: Vec<ResolvedDependency> = Vec::new();
  let mut version_lookup: HashMap<String, semver::Version> = HashMap::new();
  let options = get_output_metadata_options();

  match ops::resolve_dependencies(ws, None, &options.features, options.all_features, options.no_default_features, &[]) {
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
                Some(EncodableVersion::new(&version)),
                Some(package.manifest_path().clone())));
          },
          None => {
            resolved_dependencies.push(
              ResolvedDependency::new(&dependency, None, Some(package.manifest_path().clone()))
            );
          }
        }
      }
    },
    Err(e) => {
      println_stderr!("Could not get versions for dependencies: {:?}", e);
    }
  }

  Some(resolved_dependencies)
}

/// # override_path_dependencies
/// Replace path dependencies that have a version and don't exist on the filesystem.
///
/// ## Returns
/// Package
pub fn override_path_dependencies(package: &Package) -> Option<Package> {
  let dependencies: Vec<Dependency> = package.dependencies().iter().map(|dep| dep.clone()).collect();
  let mut new_dependencies = Vec::<Dependency>::new();
  let registry_url = "registry+".to_string() + CRATES_IO;

  // Replace for paths that don't exist.
  for dependency in dependencies {
    if dependency.source_id().is_path() {
      let url = dependency.source_id().to_url().clone();
      let path = Path::new(url.as_str());
      if path.exists() {
        new_dependencies.push(dependency.clone());
      } else {
        match SourceId::from_url(registry_url.as_str()) {
          Ok(sid) => {
            new_dependencies.push(dependency.clone_inner().set_source_id(sid).into_dependency());
          }
          Err(_) => {
            println_stderr!("Could not override dependency: {:?}", dependency.name());
          }
        }
      }
    } else {
      new_dependencies.push(dependency.clone());
    }
  }

  match Summary::new(
    package.manifest().summary().package_id().clone(),
    new_dependencies,
    package.manifest().summary().features().clone(),
  ) {
    Ok(summary) => {
      let manifest = Manifest::new(summary,
        package.manifest().targets().iter().map(|t| t.clone()).collect(),
        package.manifest().exclude().iter().map(|e| e.clone()).collect(),
        package.manifest().include().iter().map(|i| i.clone()).collect(),
        package.manifest().links().as_ref().map(|l| l.to_string()),
        package.manifest().metadata().clone(),
        package.manifest().profiles().clone(),
        package.manifest().publish(),
        package.manifest().replace().iter().map(|r| r.clone()).collect(),
        package.manifest().workspace_config().clone()
      );

      Some(Package::new(manifest, package.manifest_path().clone()))
    },
    Err(e) => {
      println_stderr!("Could not create new package while override dependencies: {:?}", e);
      None
    }
  }
}
