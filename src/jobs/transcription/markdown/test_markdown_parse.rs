use itertools::Itertools as _;

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
> >[This is](alink)";
    let expected = vec![
        MarkdownNode::Headline(HeadlineNode::new(
            0,
            1,
            "Hello world".to_owned(),
            "# Hello world".to_owned(),
            None,
        )),
        MarkdownNode::ParagraphNode(ParagraphNode::new(1, "content".to_string(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(
            2,
            "                content".to_string(),
            None,
        )),
        MarkdownNode::Headline(HeadlineNode::new(
            3,
            1,
            "Hello World".to_owned(),
            "# Hello World".to_owned(),
            None,
        )),
        MarkdownNode::BlockStart(BlockNode::new(4, 1, None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(4, " hello?".to_string(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(5, " ".to_string(), Some(">".into()))),
        MarkdownNode::BlockStart(BlockNode::new(5, 2, None)),
        MarkdownNode::LinkNode(LinkNode::new(
            5,
            "This is".to_owned(),
            "alink".to_owned(),
            None,
        )),
        MarkdownNode::BlockEnd(BlockNode::new(5, 2, None)),
        MarkdownNode::BlockEnd(BlockNode::new(5, 1, None)),
    ];

    let res = super::parse_markdown::parse_markdown(input).unwrap();
    assert_eq!(res.len(), expected.len());
    for (idx, res) in res.iter().enumerate() {
        assert_eq!(*res, expected[idx]);
    }
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
                "                content".to_string(),
                None
            )),
            MarkdownNode::BlockStart(BlockNode::new(1, 1, None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, "   ".to_string(), None)),
            MarkdownNode::Headline(HeadlineNode::new(
                1,
                1,
                "hello?".to_owned(),
                "#         hello?".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(
                2,
                "     #         hello?".to_owned(),
                Some(">".into())
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(3, "    ".to_owned(), Some(">".into()))),
            MarkdownNode::BlockStart(BlockNode::new(3, 2, None)),
            MarkdownNode::LinkNode(LinkNode::new(
                3,
                "This is".to_owned(),
                "alink".to_owned(),
                None
            )),
            MarkdownNode::BlockEnd(BlockNode::new(3, 2, None)),
            MarkdownNode::BlockEnd(BlockNode::new(3, 1, None)),
            MarkdownNode::Headline(HeadlineNode::new(
                4,
                1,
                "Test".to_owned(),
                "#                   Test".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(5, "  ".to_owned(), None)),
            MarkdownNode::LinkNode(LinkNode::new(
                5,
                "This is".to_owned(),
                "alink".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(
                5,
                "                                                    [This is](a link)".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(6, "    ".to_owned(), None)),
            MarkdownNode::LinkNode(LinkNode::new(
                6,
                "this is".to_owned(),
                "alink".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(
                7,
                "Empty line (whith whitespaces):".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(
                8,
                "                        ".to_owned(),
                None
            )),
            MarkdownNode::ParagraphNode(ParagraphNode::new(9, "".to_owned(), None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(10, "".to_owned(), None)),
        ]
    );
}
#[test]
fn test_newline() {
    let input = "\n";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![
            MarkdownNode::ParagraphNode(ParagraphNode::new(0, "".to_owned(), None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, "".to_owned(), None))
        ]
    );
    let input = r"
Hello world
";
    assert_eq!(
        super::parse_markdown::parse_markdown(input).unwrap(),
        vec![
            MarkdownNode::ParagraphNode(ParagraphNode::new(0, "".to_owned(), None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, "Hello world".to_owned(), None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(2, "".to_owned(), None))
        ]
    );
}
#[test]
fn test_not_block() {
    let input = "\
> Hello World
> > Hello World
> > hello > not hello";
    let expected = vec![
        MarkdownNode::BlockStart(BlockNode::new(0, 1, None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(0, " Hello World".to_owned(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(1, " ".to_owned(), Some(">".into()))),
        MarkdownNode::BlockStart(BlockNode::new(1, 2, None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(1, " Hello World".to_owned(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(
            2,
            " hello > not hello".to_owned(),
            Some("> >".into()),
        )),
        MarkdownNode::BlockEnd(BlockNode::new(2, 2, None)),
        MarkdownNode::BlockEnd(BlockNode::new(2, 1, None)),
    ];

    let res = super::parse_markdown::parse_markdown(input).unwrap();
    assert_eq!(res.len(), expected.len());
    for (idx, res) in res.into_iter().enumerate() {
        assert_eq!(res, expected[idx]);
    }
}
#[test]
fn test_parse_broken_links() {
    let input = "\
[This is a link(href)
[This is ](alink)
[This is] (alink)
[This is](a link)
[This is](alink";
    let expected = vec![
        MarkdownNode::ParagraphNode(ParagraphNode::new(0, "[This is a link(href)".into(), None)),
        MarkdownNode::LinkNode(LinkNode::new(1, "This is ".into(), "alink".into(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(2, "[This is] (alink)".into(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(3, "[This is](a link)".into(), None)),
        MarkdownNode::ParagraphNode(ParagraphNode::new(4, "[This is](alink".into(), None)),
    ];
    let res = super::parse_markdown::parse_markdown(&input).unwrap();
    assert_eq!(res.len(), expected.len());
    for (idx, res) in res.into_iter().enumerate() {
        assert_eq!(res, expected[idx]);
    }
}
#[test]
fn construction_test() {
    let mut inputs = vec![];

    inputs.push(super::test_data::get_test_file1());

    for input in inputs {
        let parsed = super::parse_markdown::parse_markdown(&input).unwrap();
        let res = super::parse_markdown::construct_markdown(parsed)
            .unwrap()
            .split("\n")
            .map(|x| x.to_owned())
            .collect_vec();
        let exp = input.split("\n").collect_vec();
        assert_eq!(res.len(), exp.len(), "Lengths dont match");
        for (idx, exp) in exp.into_iter().enumerate() {
            assert_eq!(&res[idx], exp, "[{}]", idx);
        }
    }
}
#[test]
fn test_persistency() {
    let input = ">>";
    let expected = ">>";
    let res = super::parse_markdown::parse_markdown(input).unwrap();
    assert_eq!(
        res,
        vec![
            MarkdownNode::BlockStart(BlockNode::new(0, 1, None)),
            MarkdownNode::BlockStart(BlockNode::new(0, 2, None)),
            MarkdownNode::BlockEnd(BlockNode::new(0, 2, None)),
            MarkdownNode::BlockEnd(BlockNode::new(0, 1, None)),
        ]
    );
    assert_eq!(
        super::parse_markdown::construct_markdown(res).unwrap(),
        expected
    );
}

#[test]
fn test_injection() {
    let input = "\
# Hello world
> lets go
> hello?
";
    let expected = "\
# Hello world
> lets go
> INJECTED
> hello?
";
    let parsed = super::parse_markdown::parse_markdown(input).unwrap();
    assert_eq!(
        parsed,
        vec![
            MarkdownNode::Headline(HeadlineNode::new(
                0,
                1,
                "Hello world".into(),
                "# Hello world".into(),
                None
            )),
            MarkdownNode::BlockStart(BlockNode::new(1, 1, None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(1, " lets go".into(), None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(2, " hello?".into(), Some(">".into()))),
            MarkdownNode::BlockEnd(BlockNode::new(2, 1, None)),
            MarkdownNode::ParagraphNode(ParagraphNode::new(3, "".into(), None)),
        ]
    )
}
