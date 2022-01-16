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

Arch-rebuild-order uses the local syncdb to build a hashmap, mapping packages
to their reverse (make) dependencies. The provided pkgnames are looked up in
the syncdb and a hashmap is built of the package provides and the pkgname
called **provides_map**. The **pkgnames** and **provides** are added to the
**to_visit** list and this then starts the iteration over every entry in the
list. During each iteration, the real package name is resolved if the provided
package comes from provides using the **provides_map**, a graph node is created
for the entry. For all reverse (make) dependencies of the entry, the dependency is added to
the **to_visit** list, a new graph node is created and added as an edge of the
pkg node. This repeats until the **to_visit** list is empty.

## DOT output

Arch-rebuild-order can generate a DOT file of the rebuild order for a given package.

```
cargo run -- -d opencolorio.dot opencolorio
dot -Tpng opencolorio.dot > opencolorio.png
```

## Limitations

* `testing` and `community-testing` repositories are not included.
* Arch-rebuild-order expects an up-to-date syncdb and does not provide warning if it is not.

## Completions

Shell completions can be created with `cargo run --bin completions` in a
directory specified by the env variable `OUT_DIR`.
