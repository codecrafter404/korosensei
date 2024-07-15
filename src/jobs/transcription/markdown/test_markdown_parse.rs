#[test]
fn test_headline_node() {
    let input = "\
# Hello world
content
                content
# Hello World
";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![]
    );
    assert!(false);
}
