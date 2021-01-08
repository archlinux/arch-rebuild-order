# Arch Linux Rebuild Order Tool

A CLI tool to determine the rebuild order of provided package(s).

## Usage

To show the rebuild order of opencolorio

```
cargo run opencolorio
```

## Requirements

- Generate a list of packages to rebuild in order for given package(s).
- Generate the build order within one second.

## Algorithm

Rebuilder uses the local syncdb to build a hashmap, mapping packages to their reverse (make)
dependencies. It adds the provided pkgnames to the **to_visit** list and iterates over each entry,
pops it to inspect and in turn adds all found reverse dependencies again to the **to_visit** list.
It repeats this cycle until the entire **to_visit** list is empty.

During this iteration process a pkg node is created in a DiGraph and for all reverse dependencies
of this package additional node are created and added as an edge of the parent pkg node.

## DOT output

Rebuilder can generate a DOT file of the rebuild order for a given package.

```
cargo run -- -d opencolorio.dot opencolorio
dot -Tpng opencolorio.dot > opencolorio.png
```

## Limitations

* `testing` and `community-testing` repositories are not included.
* Rebuilder expects an up-to-date syncdb and does not provide warning if it is not.

## Completions

Shell completions can be created with `cargo run --bin completions` in a
directory specified by the env variable `OUT_DIR`.
