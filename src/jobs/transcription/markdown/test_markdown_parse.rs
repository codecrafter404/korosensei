use crate::jobs::transcription::markdown::parse_markdown::{
    BlockNode, HeadlineNode, LinkNode, MarkdownNode, ParagraphNode,
};

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
        vec![
            MarkdownNode::ParagraphNode(ParagraphNode::new(
                0,
                "                content".to_string()
            )),
            MarkdownNode::BlockStart(BlockNode::new(1, 1)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, "   ".to_string())),
            MarkdownNode::Headline(HeadlineNode::new(
                1,
                1,
                "hello?".to_owned(),
                "#         hello?".to_owned()
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(2, "     #         hello?".to_owned())),
            MarkdownNode::ParagraphNode(ParagraphNode::new(3, "    ".to_owned())),
            MarkdownNode::BlockStart(BlockNode::new(3, 2)),
            MarkdownNode::LinkNode(LinkNode::new(3, "This is".to_owned(), "alink".to_owned())),
            MarkdownNode::BlockEnd(BlockNode::new(4, 2)),
            MarkdownNode::BlockEnd(BlockNode::new(4, 1)),
            MarkdownNode::Headline(HeadlineNode::new(
                5,
                1,
                "Test".to_owned(),
                "#                   Test".to_owned()
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(5, "  ".to_owned())),
            MarkdownNode::LinkNode(LinkNode::new(5, "This is".to_owned(), "alink".to_owned())),
            MarkdownNode::ParagraphNode(ParagraphNode::new(5, "    ".to_owned())),
        ]
    );
}
#[test]
fn test_newline() {
    let input = "\n";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![
            MarkdownNode::ParagraphNode(super::parse_markdown::ParagraphNode {
                line: 0,
                content: "".to_owned()
            }),
            MarkdownNode::ParagraphNode(super::parse_markdown::ParagraphNode {
                line: 1,
                content: "".to_owned()
            })
        ]
    );
    let input = r"
Hello world
";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![
            MarkdownNode::ParagraphNode(ParagraphNode::new(0, "".to_owned())),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, "Hello world".to_owned())),
            MarkdownNode::ParagraphNode(ParagraphNode::new(2, "".to_owned()))
        ]
    );
}
