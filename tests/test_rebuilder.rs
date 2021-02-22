use rstest::rstest;
use tempfile::TempDir;

pub mod fixtures;

use fixtures::{
    dependency_cycle, dependency_depth, invalid_dbpath, multiple_deps, no_reverse_deps,
    provides_make_depends, reverse_deps, reverse_make_deps, Package,
};

#[rstest]
#[should_panic]
fn test_invalid_dbpath(invalid_dbpath: (Vec<String>, Option<String>)) {
    let pkgnames = invalid_dbpath.0;
    let dbpath = invalid_dbpath.1;
    arch_rebuild_order::run(pkgnames, dbpath, vec![], None).unwrap();
}

/// A package without any reverse dependencies should only print the given package
#[rstest]
fn test_no_reverse_deps(no_reverse_deps: (Vec<Package>, Option<String>, Vec<String>, TempDir)) {
    let packages = no_reverse_deps.0;

    let res = arch_rebuild_order::run(
        vec![packages[0].name.clone()],
        no_reverse_deps.1,
        no_reverse_deps.2,
        None,
    )
    .unwrap();
    assert_eq!(packages[0], res.trim());
}

/// Given a package 'testpkg1' with a reverse dependency on 'testpkg2', the rebuild order should be
/// 'testpkg1 testpkg2'
#[rstest]
fn test_reverse_deps(reverse_deps: (Vec<String>, Option<String>, Vec<String>, TempDir)) {
    let pkgnames = reverse_deps.0;
    let pkgname = &pkgnames[0];

    let res = arch_rebuild_order::run(
        vec![pkgname.to_string()],
        reverse_deps.1,
        reverse_deps.2,
        None,
    )
    .unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(pkgnames, res_pkgs);
}

/// Given a package 'testpkg1' with a reverse make dependency on 'testpkg2', the rebuild order
/// should be 'testpkg1 testpkg2'
#[rstest]
fn test_reverse_make_deps(reverse_make_deps: (Vec<Package>, Option<String>, Vec<String>, TempDir)) {
    let packages = reverse_make_deps.0;
    let pkgname = &packages[0].name;

    let res = arch_rebuild_order::run(
        vec![pkgname.to_string()],
        reverse_make_deps.1,
        reverse_make_deps.2,
        None,
    )
    .unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(packages, res_pkgs);
}

/// Given a package 'testpkg1' which provides 'pkg1' where 'testpkg2' make depends on 'pkg1',
/// the rebuild order should be 'testpkg1 testpkg2'
#[rstest]
fn test_provides_make_depends(
    provides_make_depends: (Vec<Package>, Option<String>, Vec<String>, TempDir),
) {
    let packages = provides_make_depends.0;
    let pkgname = &packages[0].name;

    let res = arch_rebuild_order::run(
        vec![pkgname.to_string()],
        provides_make_depends.1,
        provides_make_depends.2,
        None,
    )
    .unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(packages, res_pkgs);
}

/// Given a package 'testpkg1' with a reverse dependency of 'testpkg2' and 'testpkg3', the rebuild
/// order should be 'testpkg1 'testpkg2 testpkg3'
#[rstest]
fn test_multiple_deps(multiple_deps: (Vec<Package>, Option<String>, Vec<String>, TempDir)) {
    let packages = multiple_deps.0;
    let pkgname = &packages[0];

    let res = arch_rebuild_order::run(
        vec![pkgname.to_string()],
        multiple_deps.1,
        multiple_deps.2,
        None,
    )
    .unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(packages[0], res_pkgs[0]);
}

/// Given a package 'testpkg1' with a reverse dependency of 'testpkg2' which has a reverse
/// dependency on 'testpkg3' the rebuild order should be 'testpkg1 'testpkg2 testpkg3'
#[rstest]
fn test_dependency_depth(dependency_depth: (Vec<Package>, Option<String>, Vec<String>, TempDir)) {
    let packages = dependency_depth.0;
    let pkgname = &packages[0];

    let res = arch_rebuild_order::run(
        vec![pkgname.to_string()],
        dependency_depth.1,
        dependency_depth.2,
        None,
    )
    .unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(packages[0], res_pkgs[0]);
}

/// Given a package 'testpkg1' with a dependency on 'testpkg2' and 'testpkg2' having a dependency
/// on 'testpkg1'. Providing 'testpkg1' should return 'testpkg1 testpkg2'
/// TODO: There should be a warning that there is a dependency cycle and the dep cycle should be
/// broken #3.
#[rstest]
fn test_dependency_cycle(dependency_cycle: (Vec<Package>, Option<String>, Vec<String>, TempDir)) {
    let packages = dependency_cycle.0;
    let pkgname = &packages[0];

    let res = arch_rebuild_order::run(
        vec![pkgname.to_string()],
        dependency_cycle.1,
        dependency_cycle.2,
        None,
    )
    .unwrap();
    let res_pkgs: Vec<&str> = res.trim().split_ascii_whitespace().collect();
    assert_eq!(packages[0], res_pkgs[0]);
}
