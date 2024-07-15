use color_eyre::eyre::{eyre, OptionExt};
use itertools::Itertools;

use super::char_stream::CharStream;

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

/// Consumes newline
#[derive(Debug, Clone, PartialEq, Eq)]
struct HeadlineNode {
    line: usize,
    level: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
    original: String,
}
impl ParsableMarkdownNode for HeadlineNode {
    fn parse(content: &str, line: usize, original: &str) -> color_eyre::Result<Self>
    where
        Self: Sized,
    {
        let (_, hash, text) = lazy_regex::regex_captures!(r"^\s{0,3}(#{1,})\s{1,}(.*)$", &content)
            .ok_or_eyre(format!("Expected to match a headline, got '{}'", content))?;
        Ok(HeadlineNode {
            content: text.to_string(),
            line,
            level: hash.len(),
            original: content.to_string(),
        })
    }

    fn construct(&self) -> String {
        self.original.clone()
    }
}
/// Consumes newline
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphNode {
    line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockNode {
    /// at this line is the first or last '>'
    line: usize,
    /// nested level
    level: usize,
}
impl BlockNode {
    fn parse(content: &mut CharStream, line: usize, level: usize) -> color_eyre::Result<BlockNode> {
        if content.take(1) != vec!['>'] {
            return Err(eyre!("Expected to get Block starting with '>'"));
        }
        Ok(BlockNode {
            line: level,
            level: line,
        })
    }

    fn construct(&self) -> String {
        return ">".to_string();
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkNode {
    line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
    /// can be "" or only whitespace etc.
    href: String,
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
    for (idx, original_line) in content.split("\n").enumerate() {
        let mut line = original_line.to_string();

        while let Some(pre_test) = pre.iter().next() {
            if line
                .chars()
                .filter(|x| !x.is_whitespace())
                .collect::<String>()
                .starts_with(pre_test)
            {
                let mut pre_test = pre_test.chars().collect_vec();

                // strip_prefix ignoring whitespace
                while let Some(x) = line.chars().next() {
                    if x.is_whitespace() {
                        line = line[1..].to_string();
                    } else if pre_test.iter().next().is_some_and(|y| *y == x) {
                        line = line[1..].to_string();
                        if pre_test.len() != 1 {
                            pre_test = pre_test[1..].to_vec();
                        }
                    }
                }
            } else {
                res.push(MarkdownNode::BlockEnd(BlockNode {
                    line: idx - 1,
                    level: pre.len(),
                }));
                pre.pop();
            }
        }

        let mut line_stream = super::char_stream::CharStream::new(&line.chars().collect_vec());

        let white_space = line_stream.take_while(|x| x.is_whitespace());

        if white_space.iter().filter(|x| **x == ' ').count() < 4
            && !white_space.iter().any(|x| *x == '\t')
        {
            println!(
                "{:?} {:?} {:?} {:?} {:?} {:?}",
                line_stream,
                idx,
                pre,
                line,
                original_line,
                line_stream.preview(1)
            );
            res.extend_from_slice(&parse_line(&mut line_stream, &line, idx)?);
        } else {
            res.push(MarkdownNode::ParagraphNode(ParagraphNode {
                line: idx,
                content: format!("{}\n", original_line),
            }));
            continue;
        }
    }

    println!("res: {:#?}", res);
    Ok(res)
}
fn parse_line(
    line_stream: &mut CharStream,
    full_line: &str,
    index: usize,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut res = Vec::new();
    if line_stream.test(|x| x == '#').is_some_and(|x| x) {
        res.push(MarkdownNode::Headline(HeadlineNode::parse(
            &full_line, index,
        )?));
    }
    if line_stream.test(|x| x == '>').is_some_and(|x| x) {
        res.push(MarkdownNode::Headline(HeadlineNode::parse(
            &full_line, index,
        )?));
    }
    Ok(res)
}
pub(crate) fn construct_markdown(nodes: Vec<MarkdownNode>) -> String {
    unimplemented!()
}
