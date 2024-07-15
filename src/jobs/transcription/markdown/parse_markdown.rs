trait ParsableMarkdownNode {
    fn parse<'a>(content: &'a str) -> (Self, &'a str)
    where
        Self: Sized; // -> (Self, left over parsing)
    fn construct(&self) -> String;
}

/// Consumes newline
#[derive(Debug, Clone)]
struct HeadlineNode {
    line: usize,
    level: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: usize,
}
/// Consumes newline
#[derive(Debug, Clone)]
struct ParagraphNode {
    line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
}

#[derive(Debug, Clone)]
struct BlockNode {
    /// at this line is the first or last '>'
    line: usize,
    /// nested level
    level: usize,
}
#[derive(Debug, Clone)]
struct LinkNode {
    line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
    /// can be "" or only whitespace etc.
    href: String,
}
#[derive(Debug, Clone)]
enum MarkdownNode {
    Headline(HeadlineNode),
    ParagraphNode(ParagraphNode),
    BlockStart(BlockNode),
    BlockEnd(BlockNode),
    LinkNode(LinkNode),
}

pub(crate) fn parse_markdown(content: String) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut content = content;

    unimplemented!()
}
pub(crate) fn construct_markdown(nodes: Vec<MarkdownNode>) -> String {
    unimplemented!()
}
