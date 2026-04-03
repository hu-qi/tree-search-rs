//! Tests for version functionality

use tree_search_rs::VERSION;

#[test]
fn test_version_constant() {
    // Verify VERSION constant is not empty and matches expected format
    assert!(!VERSION.is_empty(), "VERSION should not be empty");
    
    // Version should be in semver format (major.minor.patch)
    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(parts.len() >= 2, "VERSION should be in semver format (major.minor.patch)");
    
    // Verify each part is numeric
    for part in &parts {
        assert!(part.parse::<u32>().is_ok(), "Each version part should be numeric");
    }
}

#[test]
fn test_version_matches_cargo() {
    // VERSION should match the version in Cargo.toml
    // This is automatically set by the env!("CARGO_PKG_VERSION") macro
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}
