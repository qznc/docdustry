use docdustry::config::Config;
use std::path::PathBuf;

#[test]
fn test_get_sources_returns_current_dir_when_empty() {
    let config = Config::new();
    let sources = config.get_sources();

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0], PathBuf::from("."));
}

#[test]
fn test_push_source_dir_adds_multiple_sources() {
    let mut config = Config::new();
    let source1 = PathBuf::from("source1");
    let source2 = PathBuf::from("source2");
    let source3 = PathBuf::from("source3");

    config.push_source_dir(source1.clone());
    config.push_source_dir(source2.clone());
    config.push_source_dir(source3.clone());

    let sources = config.get_sources();

    assert_eq!(sources.len(), 3);
    assert!(sources.contains(&source1));
    assert!(sources.contains(&source2));
    assert!(sources.contains(&source3));
}
