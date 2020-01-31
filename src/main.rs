use std::collections::{VecDeque, HashMap, HashSet};

use structopt::StructOpt;
use alpm::{SigLevel, Package};
use anyhow::{Context, Result, Error};


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

fn get_reverse_deps_map(pacman: &alpm::Alpm) -> Result<HashMap<String, Vec<String>>, Error> {
    let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();
    let dbs = pacman.syncdbs();

    for db in dbs {
        for pkg in db.pkgs().context("Unable to get packages")? {
            for dep in pkg.depends() {
                reverse_deps.entry(dep.name().to_string())
                    .and_modify(|e| e.push(pkg.name().to_string() ))
                    .or_insert_with(|| vec![pkg.name().to_string()]);
            }

            for dep in pkg.makedepends() {
                reverse_deps.entry(dep.name().to_string())
                    .and_modify(|e| e.push(pkg.name().to_string() ))
                    .or_insert_with(|| vec![pkg.name().to_string()]);
            }
        }
    }

    Ok(reverse_deps)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rebuilder", about, author)]
struct Args {
    /// List of input packages
    #[structopt(min_values = 1, required = true)]
    pkgnames: Vec<String>
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let pkgnames = args.pkgnames;

    let pacman = alpm::Alpm::new(ROOT_DIR, DB_PATH).context("could not initialise pacman db")?;
    let _core = pacman.register_syncdb("core", SigLevel::NONE);
    let _extra = pacman.register_syncdb("extra", SigLevel::NONE);
    let _community = pacman.register_syncdb("community", SigLevel::NONE);
    //let _multilib = pacman.register_syncdb("mulitlib", SigLevel::NONE);
    let reverse_deps_map = get_reverse_deps_map(&pacman)?;

    for pkg in &pkgnames {
        find_package_anywhere(&pkg, &pacman)?;
    }

    let mut to_visit = VecDeque::new();
    let mut to_build = HashSet::new();
    to_visit.extend(pkgnames.iter().map(|x| x.as_str()));

    while !to_visit.is_empty() {
        let pkg = if let Some(pkg) = to_visit.pop_front() {
            pkg
        } else {
            break;
        };

        if let Some(rev_deps_for_pkg) = reverse_deps_map.get(pkg) {
            if to_build.get(&pkg.to_string()).is_none() {
                to_visit.extend(rev_deps_for_pkg.iter().map(|x| x.as_str()));
            }
            to_build.extend(rev_deps_for_pkg);
        };
    }

    dbg!(to_build.clone());
    dbg!(to_build.len());

    Ok(())
}
