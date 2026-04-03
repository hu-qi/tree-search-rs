//! Tests for version functionality

use tree_search_rs::VERSION;

#[test]
fn test_version_constant() {
    // Verify VERSION constant is not empty and matches expected format
    assert!(!VERSION.is_empty(), "VERSION should not be empty");
    
    // Version should be in semver format (major.minor.patch)
    // Handle pre-release identifiers by splitting on '-' first
    let version_without_prerelease = VERSION.split('-').next().unwrap_or(VERSION);
    let parts: Vec<&str> = version_without_prerelease.split('.').collect();
    assert!(parts.len() >= 3, "VERSION should be in semver format (major.minor.patch)");
    
    // Verify each part is numeric
    for part in &parts {
        assert!(part.parse::<u32>().is_ok(), "Each version part should be numeric");
    }
}

#[test]
fn test_version_matches_cargo() {
    // This test serves as a sanity check that VERSION is accessible from integration tests.
    // While this may appear to be a tautology (since VERSION is defined using env!("CARGO_PKG_VERSION")),
    // it verifies that the constant is properly exported and accessible in the test environment.
    // This provides value by ensuring the public API is correctly set up for external consumers.
    assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
}
