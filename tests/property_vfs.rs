//! Property tests for VFS path validation rules.

#[path = "support/rng.rs"]
mod rng;

use aether_kernel::vfs::path::validate_path;
use aether_kernel::vfs::MAX_PATH_LEN;
use rng::for_each_case;

const COMPONENTS: &[&str] = &["init", "etc", "dev", "tmp", "a", "bin", "config"];

#[test]
fn generated_absolute_paths_are_accepted() {
    assert!(validate_path("/").is_ok());
    for component in COMPONENTS {
        let path = format!("/{component}");
        assert!(path.len() <= MAX_PATH_LEN);
        assert!(validate_path(&path).is_ok(), "path: {path}");
    }
    for a in COMPONENTS {
        for b in COMPONENTS {
            let path = format!("/{a}/{b}");
            assert!(path.len() <= MAX_PATH_LEN);
            assert!(validate_path(&path).is_ok(), "path: {path}");
        }
    }
}

#[test]
fn paths_with_parent_components_are_rejected() {
    for_each_case(128, |rng, _| {
        let a = COMPONENTS[rng.next_bounded(COMPONENTS.len() as u64) as usize];
        let b = COMPONENTS[rng.next_bounded(COMPONENTS.len() as u64) as usize];
        let path = format!("/{a}/../{b}");
        assert!(validate_path(&path).is_err(), "path: {path}");
    });
}

#[test]
fn relative_paths_are_rejected() {
    for component in COMPONENTS {
        assert!(validate_path(component).is_err());
        assert!(validate_path(&format!("{component}/nested")).is_err());
    }
}

#[test]
fn empty_path_is_rejected() {
    assert!(validate_path("").is_err());
}
