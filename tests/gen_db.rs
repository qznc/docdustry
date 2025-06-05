use docdustry::gen_db::parse_markdown_to_meta;

#[test]
fn test_parse_markdown_to_meta() {
    let raw = String::from(
        r#"# Test

This is *Markdown*

```docdustry-docmeta
id: my_test
status: accepted
tag: random_tag
tag: foo
```

See [link](did:foo) test.
"#,
    );
    let meta = parse_markdown_to_meta(&raw);
    assert_eq!(meta.did, "my_test");
    assert_eq!(meta.title, "Test");
    assert_eq!(meta.tags[0], "random_tag");
    assert_eq!(meta.tags[1], "foo");
    assert_eq!(meta.relations[0].from, "my_test");
    assert_eq!(meta.relations[0].verb, "links");
    assert_eq!(meta.relations[0].to, "foo");
}
