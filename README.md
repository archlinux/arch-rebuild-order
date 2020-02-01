# Arch Linux Rebuilder

Tool to determine the rebuild order of provided package(s).

## Requirements

- generate a list of packages to rebuild in order for given package(s)
- generate the build order within one second

## Algorithm

The rebuilder program uses the local syncdb's to build a hashmap consisting a
mapping of packages to it's reverse (make) dependencies. Now the programs adds
the provided pkgnames to the **to_visit** list and starts to walk over the
**to_visit** list pops a package to inspect from the list and adds the found
reverse dependencies to the **to_visit** list and continues this until
**to_visit** is empty.

During the iteration the pkg node is create to a DiGraph and for the reverse
dependencies of this package a node is created and added as an edge of the
pkg node.

## Limitations

* the testing/community-testing repositories are not included
* the script expects an updated syncdb and does not warn if they are old
