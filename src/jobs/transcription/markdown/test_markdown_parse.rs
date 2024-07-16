use crate::jobs::transcription::markdown::parse_markdown::{MarkdownNode, ParagraphNode};

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
#[test]
fn test_white_space() {
    let input = r"                content
>   #         hello?
>     #         hello?
>    >[This is](alink)
#                   Test
  [This is](alink)                                                    [This is](a link)
    [this is](alink)
Empty line (whith whitespaces):
                        

";
    assert_eq!(input.chars().next(), Some(' '));
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![]
    );
    assert!(false);
}
#[test]
fn test_newline() {
    let input = "\n";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![super::parse_markdown::MarkdownNode::ParagraphNode(
            super::parse_markdown::ParagraphNode {
                line: 0,
                content: "\n".to_owned()
            }
        )]
    );
    let input = r"
Hello world
";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![
            MarkdownNode::ParagraphNode(ParagraphNode::new(0, "\n".to_owned())),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, "Hello world".to_owned())),
            MarkdownNode::ParagraphNode(ParagraphNode::new(2, "\n".to_owned()))
        ]
    );
}
