use alpm::{Package, SigLevel};
use petgraph::dot::{Config, Dot};
use petgraph::graph::DiGraph;
use petgraph::visit::DfsPostOrder;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};

const ROOT_DIR: &str = "/";
const DB_PATH: &str = "/var/lib/pacman/";

/// Attempt to find any match of a package in the syncdb.
fn find_package_anywhere<'a>(
    pkgname: &str,
    pacman: &'a alpm::Alpm,
) -> Result<Package<'a>, alpm::Error> {
    let dbs = pacman.syncdbs();
    for db in dbs {
        let maybe_pkg = db.pkg(pkgname);
        if maybe_pkg.is_ok() {
            return maybe_pkg;
        }
    }
    Err(alpm::Error::PkgNotFound)
}

/// Retrieve a HashMap of all reverse dependencies.
fn get_reverse_deps_map(
    pacman: &alpm::Alpm,
) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();
    let dbs = pacman.syncdbs();

    for db in dbs {
        if db.pkgs().is_err() {
            eprintln!("Unable to get packages from sync db {}", db.name());
        }
        for pkg in db.pkgs()? {
            for dep in pkg.depends() {
                reverse_deps
                    .entry(dep.name().to_string())
                    .and_modify(|e| e.push(pkg.name().to_string()))
                    .or_insert_with(|| vec![pkg.name().to_string()]);
            }

            for dep in pkg.makedepends() {
                reverse_deps
                    .entry(dep.name().to_string())
                    .and_modify(|e| e.push(pkg.name().to_string()))
                    .or_insert_with(|| vec![pkg.name().to_string()]);
            }
        }
    }

    Ok(reverse_deps)
}

/// Write a given DiGraph to a given file using a buffered writer.
fn write_dotfile(filename: String, graph: DiGraph<&str, u16>) -> Result<(), Box<dyn Error>> {
    let dotgraph = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    let file = File::create(filename)?;
    let mut bufw = BufWriter::new(file);
    bufw.write_all(dotgraph.to_string().as_bytes())?;

    Ok(())
}

/// Run rebuilder, returning the rebuild order of provided package(s).
pub fn run(
    pkgnames: Vec<String>,
    dbpath: Option<String>,
    dotfile: Option<String>,
) -> Result<String, Box<dyn Error>> {
    let pacman = match dbpath {
        Some(path) => alpm::Alpm::new(ROOT_DIR, &path),
        None => alpm::Alpm::new(ROOT_DIR, DB_PATH),
    };

    if pacman.is_err() {
        eprintln!("Could not initialize pacman db");
    }
    let pacman = pacman?;

    let _core = pacman.register_syncdb("core", SigLevel::DATABASE_OPTIONAL);
    let _extra = pacman.register_syncdb("extra", SigLevel::DATABASE_OPTIONAL);
    let _community = pacman.register_syncdb("community", SigLevel::DATABASE_OPTIONAL);
    let _multilib = pacman.register_syncdb("multilib", SigLevel::DATABASE_OPTIONAL);
    let reverse_deps_map = get_reverse_deps_map(&pacman)?;

    for pkg in &pkgnames {
        find_package_anywhere(&pkg, &pacman)?;
    }

    let mut graph = DiGraph::<&str, u16>::new();

    let mut to_visit = VecDeque::new();
    let mut to_build = HashSet::new();
    to_visit.extend(pkgnames.iter().map(|x| x.as_str()));

    let mut cache_node = HashMap::new();

    while !to_visit.is_empty() {
        let pkg = if let Some(pkg) = to_visit.pop_front() {
            pkg
        } else {
            break;
        };

        let root = *cache_node.entry(pkg).or_insert_with(|| graph.add_node(pkg));

        if let Some(rev_deps_for_pkg) = reverse_deps_map.get(pkg) {
            if to_build.get(&pkg.to_string()).is_none() {
                to_visit.extend(rev_deps_for_pkg.iter().map(|x| x.as_str()));
            }

            for rev_dep in rev_deps_for_pkg {
                let depnode = *cache_node
                    .entry(rev_dep)
                    .or_insert_with(|| graph.add_node(rev_dep));
                if !graph.contains_edge(root, depnode) {
                    graph.add_edge(root, depnode, 1);
                }
            }
            to_build.extend(rev_deps_for_pkg);
        };
    }

    let mut rebuild_order_packages = Vec::new();
    for pkg in &pkgnames {
        if let Some(pkgname) = cache_node.get(pkg.as_str()) {
            let mut bfs = DfsPostOrder::new(&graph, *pkgname);

            while let Some(nx) = bfs.next(&graph) {
                let node = graph[nx];
                rebuild_order_packages.push(node);
            }
        }
    }

    // Reverse the rebuilder order as DfsPostOrder starts with the first pkgname and therefore
    // shows it as last package
    rebuild_order_packages.reverse();

    let mut output = String::new();
    for elem in rebuild_order_packages.iter() {
        output.push_str(elem);
        output.push(' ');
    }

    if let Some(filename) = dotfile {
        if let Err(e) = write_dotfile(filename, graph) {
            eprintln!("Could not write to file");
            return Err(e);
        }
    }

    Ok(output)
}
