use std::collections::{VecDeque, HashMap};

use clap::{Arg, App};
use alpm::{SigLevel, Package};
use anyhow::{Context, Result, Error};
use rayon::prelude::*;

const ROOT_DIR: &str = "/";
const DB_PATH: &str = "/var/lib/pacman/";

fn find_package_anywhere<'a>(pkgname: &str, pacman: &'a alpm::Alpm) -> Result<Package<'a>, alpm::Error> {

    let dbs = pacman.syncdbs();
    for db in dbs {
        let maybe_pkg = db.pkg(pkgname);
        if maybe_pkg.is_ok() {
            return maybe_pkg;
        }
    };
    Err(alpm::Error::PkgNotFound)
}

fn get_reverse_deps<'a>(pkgname: &str, pacman: &'a alpm::Alpm) -> Result<Vec<String>, Error> {
    let mut reverse_deps = vec![];
    let dbs = pacman.syncdbs();
    for db in dbs {
        reverse_deps.extend(
            db.pkgs().context("Unable to get packages")?.filter(
                |pkg| pkg.depends().any(|dep| dep.name() == pkgname) || pkg.makedepends().any(|dep| dep.name() == pkgname)
            ).map(|pkg| pkg.name().to_string()));
    }
    Ok(reverse_deps)
}

fn main() -> Result<()> {
    let matches = App::new("genrebuild")
                          .version("0.1")
                          .author("Jelle van der Waa <jelle@vdwaa.nl>")
                          .about("genrebuild")
                          .arg(Arg::with_name("pkgname")
                               .help("package name")
                               .required(true))
                          .get_matches();

    let pkgname = matches.value_of("pkgname").unwrap();

    let pacman = alpm::Alpm::new(ROOT_DIR, DB_PATH).context("could not initialise pacman db")?;
    let core = pacman.register_syncdb("core", SigLevel::NONE);
    let extra = pacman.register_syncdb("extra", SigLevel::NONE);
    let community = pacman.register_syncdb("community", SigLevel::NONE);
    //let multilib = pacman.register_syncdb("mulitlib", SigLevel::NONE);
    let pkg = find_package_anywhere(pkgname, &pacman)?;

    let mut reverse_deps_map = HashMap::new();
    let mut to_visit = VecDeque::new();
    to_visit.push_back(pkgname.to_string());

    while !to_visit.is_empty() {
        let pkg = if let Some(pkg) = to_visit.pop_front() {
            pkg
        } else {
            break;
        };
        if !reverse_deps_map.contains_key(&pkg) {
            let reverse_deps = get_reverse_deps(&pkg, &pacman).unwrap_or_default();
            reverse_deps_map.insert(pkg.to_string(), reverse_deps.clone());
            to_visit.par_extend(reverse_deps);
        }
    }

    //let res = get_reverse_deps(pkg.name(), &pacman);
    dbg!(reverse_deps_map);

    Ok(())
}
