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
impl HeadlineNode {
    fn parse(content: &mut CharStream, line: usize, original: &str) -> color_eyre::Result<Self>
    where
        Self: Sized,
    {
        let content = content.collect().into_iter().collect::<String>();
        let (_, hash, text) = lazy_regex::regex_captures!(r"^\s{0,3}(#{1,})\s{1,}(.*)$", &content)
            .ok_or_eyre(format!("Expected to match a headline, got '{}'", content))?;
        Ok(HeadlineNode {
            content: text.to_string(),
            line,
            level: hash.len(),
            original: original.to_string(),
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
        Ok(BlockNode { line, level })
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
impl LinkNode {
    fn parse(stream: &mut CharStream, line: usize) -> color_eyre::Result<LinkNode> {
        if stream.take(1) != vec!['['] {
            return Err(eyre!("Expected to get link starting with '['"));
        }
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
        }
        Ok(LinkNode {
            line,
            content,
            href,
        })
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
                let mut to_remove = pre.clone().join("").chars().collect_vec();
                while let Some(x) = line.chars().next() {
                    if x.is_whitespace() {
                        line = line[1..].to_string();
                    } else if to_remove.iter().next().is_some_and(|y| *y == x) {
                        line = line[1..].to_string();
                        if to_remove.len() != 1 {
                            to_remove = to_remove[1..].to_vec();
                        } else {
                            break;
                        }
                    }
                }
                break;
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
            res.extend_from_slice(&parse_line(&mut line_stream, &line, idx, &mut pre)?);
        } else {
            res.push(MarkdownNode::ParagraphNode(ParagraphNode {
                line: idx,
                content: format!("{}\n", original_line),
            }));
            continue;
        }

        // Last line cleanup
        if idx + 1 == lines.len() {
            while let Some(_) = pre.iter().next() {
                res.push(MarkdownNode::BlockEnd(BlockNode {
                    line: idx - 1,
                    level: pre.len(),
                }));
                pre.pop();
            }
        }
    }

    println!("res: {:#?}", res);
    Ok(res)
}
fn parse_line(
    line_stream: &mut CharStream,
    original_line: &str,
    index: usize,
    pre: &mut Vec<String>,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut res = Vec::new();
    if line_stream.test(|x| x == '#').is_some_and(|x| x) {
        println!("-> got headline node");
        res.push(MarkdownNode::Headline(HeadlineNode::parse(
            line_stream,
            index,
            original_line,
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

    if line_stream.is_empty() {
        return Ok(res);
    }

    let mut current = line_stream.take(1);

    loop {
        let test = parse_line(line_stream, original_line, index, pre)?;
        if !test.is_empty() {
            let p = current.clone().into_iter().collect::<String>();
            println!("-> Got paragraph");
            res.push(MarkdownNode::ParagraphNode(ParagraphNode {
                line: index,
                content: p,
            }));
            res.extend_from_slice(&test);
            break;
        }
        if !line_stream.is_empty() {
            current.extend_from_slice(&line_stream.take(1));
        } else {
            break;
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
