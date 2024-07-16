use color_eyre::eyre::{eyre, OptionExt};
use itertools::Itertools;

use crate::utils::{char_stream::CharStream, string};

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
    pub fn construct(&self) -> String {
        self.content.clone()
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
        return "".to_string();
    }
    pub fn new(line: usize, level: usize) -> BlockNode {
        BlockNode { line, level }
    }
}

/// NOTE: The content will not be reparsed!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkNode {
    line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    content: String,
    /// can be "" or only whitespace etc.
    href: String,
}
impl LinkNode {
    fn parse(stream: &mut CharStream, line: usize) -> color_eyre::Result<Option<LinkNode>> {
        let bak = stream.clone();
        if stream.take(1) != vec!['['] {
            return Err(eyre!("Expected to get link starting with '['"));
        }
        let content = stream
            .take_while(|x| x != ']')
            .into_iter()
            .collect::<String>();
        let href = if stream.take(2) == vec![']', '('] {
            stream
                .take_while(|x| x != ')')
                .into_iter()
                .collect::<String>()
        } else {
            *stream = bak;
            return Ok(None);
        };

        let x = stream.take(1); // may be ')' or EOL

        if x != vec![')'] || href.contains(" ") {
            *stream = bak;
            return Ok(None);
        }

        Ok(Some(LinkNode::new(line, content, href)))
    }
    pub fn new(line: usize, content: String, href: String) -> LinkNode {
        LinkNode {
            line,
            content,
            href,
        }
    }
    pub fn construct(&self) -> String {
        format!("[{}]({})", self.content, self.href)
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
impl MarkdownNode {
    fn get_line(&self) -> usize {
        match &self {
            MarkdownNode::Headline(x) => x.line,
            MarkdownNode::ParagraphNode(x) => x.line,
            MarkdownNode::BlockStart(x) => x.line,
            MarkdownNode::BlockEnd(x) => x.line,
            MarkdownNode::LinkNode(x) => x.line,
        }
    }
    fn construct(&self) -> String {
        match &self {
            MarkdownNode::Headline(x) => x.construct(),
            MarkdownNode::ParagraphNode(x) => x.construct(),
            MarkdownNode::BlockStart(x) => x.construct(),
            MarkdownNode::BlockEnd(x) => x.construct(),
            MarkdownNode::LinkNode(x) => x.construct(),
        }
    }
}

pub(crate) fn parse_markdown(content: &str) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut pre: Vec<String> = Vec::new();
    let mut res = Vec::new();
    let lines = content.split("\n").collect_vec();
    for (idx, original_line) in lines.clone().into_iter().enumerate() {
        let mut line = original_line.to_string();

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

        let mut line_res = parse_line(&mut line_stream, &original_line, idx, &mut pre)?;
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
                res.push(MarkdownNode::BlockEnd(BlockNode::new(idx, pre.len())));
                pre.pop();
            }
        }
    }

    println!("res: {:?}", res);
    Ok(res)
}
fn parse_stream(
    line_stream: &mut CharStream,
    index: usize,
    pre: &mut Vec<String>,
    original_line: &str,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut res = Vec::new();
    if line_stream.test(|x| x == '#').is_some_and(|x| x) {
        res.push(MarkdownNode::Headline(HeadlineNode::parse(
            line_stream,
            index,
        )?));
    }
    if line_stream.test(|x| x == '>').is_some_and(|x| x) {
        println!("H: {:?}", line_stream.get_history());
        if line_stream
            .get_history()
            .iter()
            .all(|x| x.is_whitespace() || ['>'].contains(x))
        {
            res.push(MarkdownNode::BlockStart(BlockNode::parse(
                line_stream,
                index,
                pre.len() + 1,
            )?));
            pre.push(">".to_owned());
        }
    }
    if line_stream.test(|x| x == '[').is_some_and(|x| x) {
        if let Some(x) = LinkNode::parse(line_stream, index)? {
            res.push(MarkdownNode::LinkNode(x));
        }
    }

    return Ok(res);
}
fn parse_line(
    line_stream: &mut CharStream,
    original_line: &str,
    index: usize,
    pre: &mut Vec<String>,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut res = Vec::new();

    res.extend_from_slice(&parse_stream(line_stream, index, pre, original_line)?);

    let mut current = vec![];

    loop {
        //TODO: this is different when parsing headers, (they arent valid anymore after 3x'' outside of any blocks)
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
        let test = parse_stream(line_stream, index, pre, original_line)?;
        if !test.is_empty() || line_stream.is_empty() {
            // Paragraph stuff
            let p = current.clone().into_iter().collect::<String>();
            if !p.is_empty() {
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

    Ok(res)
}
pub(crate) fn construct_markdown(nodes: Vec<MarkdownNode>) -> color_eyre::Result<String> {
    let lines = nodes.into_iter().chunk_by(|x| x.get_line());
    let lines = lines
        .into_iter()
        .map(|(a, b)| (a, b.collect_vec()))
        .collect_vec();

    let mut prev = -2;
    if let Some(x) = lines.iter().find(|x| {
        prev += 1;
        (x.0 as i32 - 1) != prev
    }) {
        return Err(eyre!(format!("line {} is missing", x.0 as i32 - 1)));
    }

    let mut result = Vec::new();
    let mut pre = Vec::new();
    for (idx, line) in lines {
        let mut res = String::new();
        let mut to_pop = 0;
        for (idx, l) in line.iter().enumerate() {
            //TODO: this is too much spagetty code, therefore set the stripped property at the first element of the newline, which containes pre with accurate spacing
            if !l.construct().trim().is_empty()
                && line.iter().nth(idx + 1).is_some_and(|x| match x {
                    MarkdownNode::BlockStart(_) => true,
                    _ => false,
                })
            {
            } else {
                res = format!("{}{}", res, l.construct());
            }
            match l {
                MarkdownNode::BlockStart(_) => {
                    pre.push(">");
                }
                MarkdownNode::BlockEnd(_) => {
                    to_pop += 1;
                }
                _ => {}
            }
        }
        let mut pre_this = format!("{}", pre.join(" "));
        println!(
            "[{:3>0}] {:?} {:?} {:?}; Line: {:?}; Starts with: {:?}",
            idx,
            pre_this,
            res,
            pre,
            line,
            pre_this.chars().next().map(|x| format!("{:x}", x as u32))
        );
        if !pre_this.is_empty() && !res.starts_with(" ") {
            pre_this = format!("{} ", pre_this);
        }
        res = format!("{}{}", pre_this, res);

        result.push(res);

        for _ in 0..to_pop {
            pre.pop();
        }
    }
    Ok(result.join("\n"))
}
