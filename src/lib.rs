use alpm::{Package, SigLevel};
use anyhow::{anyhow, Result};
use error::RebuildOrderError;
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::visit::DfsPostOrder;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufWriter, Write};

pub mod args;
pub mod error;

const ROOT_DIR: &str = "/";
const DB_PATH: &str = "/var/lib/pacman/";

/// Attempt to find any match of a package in the syncdb.
fn find_package_anywhere<'a>(pkgname: &str, pacman: &'a alpm::Alpm) -> Result<Package<'a>> {
    let dbs = pacman.syncdbs();
    for db in dbs {
        if let Ok(pkg) = db.pkg(pkgname) {
            return Ok(pkg);
        }
    }
    Err(anyhow!(RebuildOrderError::PackageNotFound))
}

/// Retrieve a HashMap of all reverse dependencies.
fn get_reverse_deps_map(
    pacman: &alpm::Alpm,
    with_check_depends: bool,
) -> HashMap<String, HashSet<String>> {
    let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new();
    let dbs = pacman.syncdbs();

    for db in dbs {
        for pkg in db.pkgs() {
            for dep in pkg.depends() {
                reverse_deps
                    .entry(dep.name().to_string())
                    .and_modify(|e| {
                        e.insert(pkg.name().to_string());
                    })
                    .or_insert_with(|| {
                        let mut modify = HashSet::new();
                        modify.insert(pkg.name().to_string());
                        modify
                    });
            }

            for dep in pkg.makedepends() {
                reverse_deps
                    .entry(dep.name().to_string())
                    .and_modify(|e| {
                        e.insert(pkg.name().to_string());
                    })
                    .or_insert_with(|| {
                        let mut modify = HashSet::new();
                        modify.insert(pkg.name().to_string());
                        modify
                    });
            }

            if with_check_depends {
                for dep in pkg.checkdepends() {
                    reverse_deps
                        .entry(dep.name().to_string())
                        .and_modify(|e| {
                            e.insert(pkg.name().to_string());
                        })
                        .or_insert_with(|| {
                            let mut modify = HashSet::new();
                            modify.insert(pkg.name().to_string());
                            modify
                        });
                }
            }
        }
    }

    reverse_deps
}

/// Write a given DiGraph to a given file using a buffered writer.
fn write_dotfile(filename: String, graph: DiGraph<&str, u16>) -> Result<()> {
    let dotgraph = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    let file = File::create(filename)?;
    let mut bufw = BufWriter::new(file);
    bufw.write_all(dotgraph.to_string().as_bytes())?;

    Ok(())
}

/// Run arch-rebuild-order, returning the rebuild order of provided package(s).
pub fn run(
    pkgnames: Vec<String>,
    dbpath: Option<String>,
    repos: Vec<String>,
    dotfile: Option<String>,
    no_reverse_depends: bool,
    with_check_depends: bool,
) -> Result<String> {
    let pacman = match dbpath {
        Some(path) => alpm::Alpm::new(ROOT_DIR, &path),
        None => alpm::Alpm::new(ROOT_DIR, DB_PATH),
    }
    .map_err(RebuildOrderError::PacmanDbInit)?;

    for repo in repos {
        let _repo = pacman.register_syncdb(repo, SigLevel::DATABASE_OPTIONAL);
    }

    let reverse_deps_map = get_reverse_deps_map(&pacman, with_check_depends);
    let mut provides = Vec::new();
    let mut provides_map = HashMap::new();

    for pkg in &pkgnames {
        let repopkg = find_package_anywhere(pkg, &pacman)?;
        for provide in repopkg.provides() {
            provides.push(provide.name());
            provides_map.insert(provide.name(), repopkg.name());
        }
    }

    let mut graph = DiGraph::<&str, u16>::new();

    let mut to_visit = VecDeque::new();
    let mut to_build = HashSet::new();

    to_visit.extend(pkgnames.iter().map(|x| x.as_str()));
    to_visit.extend(provides.iter());

    let mut cache_node: HashMap<&str, petgraph::graph::NodeIndex> = HashMap::new();

    while !to_visit.is_empty() {
        let pkg = if let Some(pkg) = to_visit.pop_front() {
            pkg
        } else {
            break;
        };

        // Resolve the provided package to the real package as provided packages are not real
        // packages in the Arch Linux repository.
        let rootpkg = provides_map.get(&pkg).unwrap_or(&pkg);
        let root = *cache_node
            .entry(rootpkg)
            .or_insert_with(|| graph.add_node(rootpkg));

        if let Some(rev_deps_for_pkg) = reverse_deps_map.get(pkg) {
            if to_build.get(&pkg.to_string()).is_none() {
                to_visit.extend(rev_deps_for_pkg.iter().map(|x| x.as_str()));
            }

            let mut rev_deps_for_pkg_vec = rev_deps_for_pkg.iter().collect::<Vec<_>>();
            rev_deps_for_pkg_vec.sort();

            for rev_dep in rev_deps_for_pkg_vec {
                let depnode = *cache_node
                    .entry(rev_dep.as_str())
                    .or_insert_with(|| graph.add_node(rev_dep));
                if !graph.contains_edge(root, depnode) {
                    graph.add_edge(root, depnode, 1);
                }
            }
            to_build.extend(rev_deps_for_pkg);
        };
    }

    // Visit nodes in our graph in a depth-first-search adding nodes in post-order. The provided
    // packages are added first to the stack.
    let mut rebuild_order_packages = Vec::new();
    let mut bfs = DfsPostOrder::empty(&graph);
    bfs.stack.extend(
        pkgnames
            .iter()
            .filter_map(|pkg| cache_node.get(pkg.as_str())),
    );

    while let Some(nx) = bfs.next(&graph) {
        let node = graph[nx];
        rebuild_order_packages.push(node);
    }

    // Reverse the rebuild order as DfsPostOrder starts with the first pkgname and therefore
    // shows it as last package
    rebuild_order_packages.reverse();

    // We only retain the packages we want to when using `--no-reverse_depends`
    // This logic is hard to parse because retain is an inverse filter,
    // thus we use the negated form of: no_reverse_depends && !pkgnames.contains(&pkg.to_string()
    rebuild_order_packages.retain(|pkg| !no_reverse_depends || pkgnames.contains(&pkg.to_string()));

    if let Some(filename) = dotfile {
        write_dotfile(filename, graph)?;
    }

    Ok(rebuild_order_packages.join(" "))
}
