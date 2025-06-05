use docdustry::serve::parse_query;

#[test]
fn test_parse_query_extracts_search_term() {
    let query = "s=test_search&other=value";
    let result = parse_query(query);
    assert_eq!(result, Some("test_search"));
}

#[test]
fn test_parse_query_handles_single_parameter() {
    let query = "s=hello";
    let result = parse_query(query);
    assert_eq!(result, Some("hello"));
}

#[test]
fn test_parse_query_returns_none_for_missing_parameter() {
    let query = "other=value&another=param";
    let result = parse_query(query);
    assert_eq!(result, None);
}

#[test]
fn test_parse_query_handles_empty_query() {
    let query = "";
    let result = parse_query(query);
    assert_eq!(result, None);
}

#[test]
fn test_parse_query_handles_malformed_query() {
    let query = "invalidquery";
    let result = parse_query(query);
    assert_eq!(result, None);
}

#[test]
fn test_parse_query_finds_s_parameter_among_many() {
    let query = "param1=value1&s=search_term&param2=value2&param3=value3";
    let result = parse_query(query);
    assert_eq!(result, Some("search_term"));
}