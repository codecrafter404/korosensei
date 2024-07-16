use color_eyre::eyre::{eyre, OptionExt};
use itertools::Itertools;

use crate::utils::{char_stream::CharStream, string};

trait ParsableMarkdownNode {
    fn parse(content: &str, line: usize) -> color_eyre::Result<Self>
    where
        Self: Sized; // -> (Self, left over parsing)
    fn construct(&self) -> String;
}
trait PartialParsableMarkdownNode {
    fn parse(content: &mut CharStream, line: usize) -> color_eyre::Result<Self>
    where
        Self: Sized; // -> (Self, left over parsing)
    fn construct(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlineNode {
    pub line: usize,
    pub level: usize,
    /// can only whitespace etc. (also linebreaks)
    pub content: String,
    pub original: String,
}
impl HeadlineNode {
    fn parse(content: &mut CharStream, line: usize) -> color_eyre::Result<Self>
    where
        Self: Sized,
    {
        let original = content.prev_collect().into_iter().collect();
        let content = content.collect().into_iter().collect::<String>();
        let (_, hash, text) = lazy_regex::regex_captures!(r"^\s{0,3}(#{1,})\s{1,}(.*)$", &content)
            .ok_or_eyre(format!("Expected to match a headline, got '{}'", content))?;
        Ok(HeadlineNode::new(
            line,
            hash.len(),
            text.to_string(),
            original,
        ))
    }

    fn construct(&self) -> String {
        self.original.clone()
    }
    pub fn new(line: usize, level: usize, content: String, original: String) -> HeadlineNode {
        HeadlineNode {
            line,
            level,
            content,
            original,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphNode {
    pub line: usize,
    /// can be only whitespace etc. (also linebreaks)
    pub content: String,
}
impl ParagraphNode {
    pub fn new(line: usize, content: String) -> ParagraphNode {
        ParagraphNode { line, content }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockNode {
    /// at this line is the first or last '>'
    pub line: usize,
    /// nested level
    pub level: usize,
}
impl BlockNode {
    fn parse(content: &mut CharStream, line: usize, level: usize) -> color_eyre::Result<BlockNode> {
        if content.take(1) != vec!['>'] {
            return Err(eyre!("Expected to get Block starting with '>'"));
        }
        Ok(BlockNode::new(line, level))
    }

    fn construct(&self) -> String {
        return ">".to_string();
    }
    pub fn new(line: usize, level: usize) -> BlockNode {
        BlockNode { line, level }
    }
}
/// WARNING: The LinkNode is not properly implemented and therefore cant handle errors at all
/// using this with broken input may lead to undefined behavior, as the parser will try to make a link out of it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkNode {
    line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
    /// can be "" or only whitespace etc.
    href: String,
}
impl LinkNode {
    fn parse(stream: &mut CharStream, line: usize) -> color_eyre::Result<LinkNode> {
        if stream.take(1) != vec!['['] {
            return Err(eyre!("Expected to get link starting with '['"));
        }
        println!("Test: {:?}; ", stream.test_while(|x| x != ']'));
        let content = stream
            .take_while(|x| x != ']')
            .into_iter()
            .collect::<String>();
        let mut href = String::new();
        if stream.take(2) == vec![']', '('] {
            href = stream
                .take_while(|x| x != ')')
                .into_iter()
                .collect::<String>();

            let _ = stream.take(1); // may be ')' or EOL
        } else {
            log::info!("Link on line {} doesn't have ']' or '('", line);
            println!("Link on line {} doesn't have ']' or '('", line);
        }
        Ok(LinkNode::new(line, content, href))
    }
    pub fn new(line: usize, content: String, href: String) -> LinkNode {
        LinkNode {
            line,
            content,
            href,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownNode {
    Headline(HeadlineNode),
    ParagraphNode(ParagraphNode),
    BlockStart(BlockNode),
    BlockEnd(BlockNode),
    LinkNode(LinkNode),
}

pub(crate) fn parse_markdown(content: &str) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut pre: Vec<String> = Vec::new();
    let mut res = Vec::new();
    let lines = content.split("\n").collect_vec();
    for (idx, original_line) in lines.clone().into_iter().enumerate() {
        let mut line = original_line.to_string();

        println!("[{:2>0}] '{}', {:?}", idx, line, pre);
        while !pre.is_empty() {
            if line
                .chars()
                .filter(|x| !x.is_whitespace())
                .collect::<String>()
                .starts_with(&pre.join(""))
            {
                line = string::strip_prefix_with_whitespace(
                    &line,
                    &pre.clone().join("").chars().collect::<String>(),
                )
                .to_string();
                break;
            } else {
                res.push(MarkdownNode::BlockEnd(BlockNode::new(idx - 1, pre.len())));
                pre.pop();
            }
        }

        let mut line_stream = CharStream::new(&line.chars().collect_vec());

        let mut line_res = parse_line(&mut line_stream, &line, idx, &mut pre, false)?;
        if line_res.is_empty() {
            // newline
            line_res.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                idx,
                "".to_owned(),
            )));
        }

        res.extend_from_slice(&line_res);

        // Last line cleanup
        if idx + 1 == lines.len() {
            // close all blocks
            while let Some(_) = pre.iter().next() {
                res.push(MarkdownNode::BlockEnd(BlockNode::new(idx - 1, pre.len())));
                pre.pop();
            }
        }
    }

    println!("res: {:#?}", res);
    Ok(res)
}
fn parse_stream(
    line_stream: &mut CharStream,
    index: usize,
    pre: &mut Vec<String>,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut res = Vec::new();
    if line_stream.test(|x| x == '#').is_some_and(|x| x) {
        println!("-> got headline node");
        res.push(MarkdownNode::Headline(HeadlineNode::parse(
            line_stream,
            index,
        )?));
    }
    if line_stream.test(|x| x == '>').is_some_and(|x| x) {
        println!("-> got block start node");
        res.push(MarkdownNode::BlockStart(BlockNode::parse(
            line_stream,
            index,
            pre.len() + 1,
        )?));
    }
    if line_stream.test(|x| x == '[').is_some_and(|x| x) {
        println!("-> got link node");
        res.push(MarkdownNode::LinkNode(LinkNode::parse(line_stream, index)?));
    }

    return Ok(res);
}
fn parse_line(
    line_stream: &mut CharStream,
    original_line: &str,
    index: usize,
    pre: &mut Vec<String>,
    test_only: bool,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    println!("=> testing char {:?}", line_stream.preview(1));
    let mut res = Vec::new();

    res.extend_from_slice(&parse_stream(line_stream, index, pre)?);

    let mut current = line_stream.take(1);

    loop {
        println!("?> current: {:?}", current);
        if (current.len() > 2 && current.iter().rev().collect::<String>().starts_with("     ")) // last 5 chars are spaces
            || current.iter().last().is_some_and(|x| *x == '\t')
        // or is tab
        {
            // too much indentation, the leftover chars are now part of this paragraph
            current.extend_from_slice(&line_stream.collect());
            res.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                index,
                current.into_iter().collect(),
            )));
            break;
        }
        let test = parse_stream(line_stream, index, pre)?;
        if !test.is_empty() || line_stream.is_empty() {
            // Paragraph stuff
            let p = current.clone().into_iter().collect::<String>();
            if !p.is_empty() {
                println!("-> Got paragraph");
                res.push(MarkdownNode::ParagraphNode(ParagraphNode::new(index, p)));
            }

            // append
            res.extend_from_slice(&test);

            current = vec![]; // allows for multiple links in one line etc
        }
        if line_stream.is_empty() {
            break;
        } else {
            current.extend_from_slice(&line_stream.take(1));
        }
    }
    if res
        .iter()
        .find(|x| match x {
            MarkdownNode::BlockStart(_) => true,
            _ => false,
        })
        .is_some()
    {
        pre.push(">".to_string());
    }

    Ok(res)
}
pub(crate) fn construct_markdown(nodes: Vec<MarkdownNode>) -> String {
    unimplemented!()
}
