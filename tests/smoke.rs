use tiny_ott::greet;

#[test]
fn greet_includes_name() {
    assert!(greet("world").contains("world"));
}
