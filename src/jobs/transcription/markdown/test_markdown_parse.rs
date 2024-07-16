#[test]
fn test_headline_node() {
    let input = "\
# Hello world
content
                content
# Hello World
> hello?
> >[This is](a link)";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![]
    );
    assert!(false);
}
