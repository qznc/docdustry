use docdustry::gen_html::Doc;
use std::path::PathBuf;

#[test]
fn test_doc_new_creates_with_correct_defaults() {
    let src_path_base = PathBuf::from("test_base");
    let src_path_rel = PathBuf::from("test_rel.md");

    let doc = Doc::new(src_path_base.clone(), src_path_rel.clone());

    assert_eq!(doc.src_path_rel, src_path_rel);
    assert_eq!(doc.title, String::new());
    assert_eq!(doc.links.len(), 0);
    assert_eq!(doc.tags.len(), 0);
    assert_eq!(doc.did, String::new());
    assert_eq!(doc.status, String::new());
    assert_eq!(doc.url, String::new());
    assert_eq!(doc.includes.len(), 0);
    assert_eq!(doc.raw, String::new());
    assert_eq!(doc.media.len(), 0);
}

#[test]
fn test_doc_rel_url_generates_proper_url() {
    let src_path_base = PathBuf::from("test_base");
    let src_path_rel = PathBuf::from("docs/test.md");
    let mut doc = Doc::new(src_path_base, src_path_rel);

    doc.did = "test_doc".to_string();

    let url = doc.rel_url();

    // Should generate a relative URL based on the document
    assert!(!url.is_empty());
    assert!(url.ends_with(".html"));
}
