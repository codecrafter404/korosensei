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
    pub stripped: Option<String>,
}
impl HeadlineNode {
    fn parse(content: &mut CharStream<char>, line: usize) -> color_eyre::Result<Self>
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
            None,
        ))
    }

    fn construct(&self) -> String {
        self.original.clone()
    }
    pub fn new(
        line: usize,
        level: usize,
        content: String,
        original: String,
        stripped: Option<String>,
    ) -> HeadlineNode {
        HeadlineNode {
            line,
            level,
            content,
            original,
            stripped,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphNode {
    pub line: usize,
    /// can be only whitespace etc. (also linebreaks)
    pub content: String,
    pub stripped: Option<String>,
}
impl ParagraphNode {
    pub fn new(line: usize, content: String, stripped: Option<String>) -> ParagraphNode {
        ParagraphNode {
            line,
            content,
            stripped,
        }
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
    pub stripped: Option<String>,
}
impl BlockNode {
    fn parse(
        content: &mut CharStream<char>,
        line: usize,
        level: usize,
    ) -> color_eyre::Result<BlockNode> {
        if content.take(1) != vec!['>'] {
            return Err(eyre!("Expected to get Block starting with '>'"));
        }
        Ok(BlockNode::new(line, level, None))
    }

    fn construct(&self) -> String {
        return ">".to_string();
    }
    pub fn new(line: usize, level: usize, stripped: Option<String>) -> BlockNode {
        BlockNode {
            line,
            level,
            stripped,
        }
    }
}

/// NOTE: The content will not be reparsed!
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkNode {
    pub line: usize,
    /// can be "" or only whitespace etc. (also linebreaks)
    pub content: String,
    /// can be "" or only whitespace etc.
    pub href: String,
    pub stripped: Option<String>,
}
impl LinkNode {
    fn parse(stream: &mut CharStream<char>, line: usize) -> color_eyre::Result<Option<LinkNode>> {
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

        Ok(Some(LinkNode::new(line, content, href, None)))
    }
    pub fn new(line: usize, content: String, href: String, stripped: Option<String>) -> LinkNode {
        LinkNode {
            line,
            content,
            href,
            stripped,
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
            MarkdownNode::BlockEnd(x) => String::new(),
            MarkdownNode::LinkNode(x) => x.construct(),
        }
    }
    fn set_stripped(&mut self, stripped: Option<String>) {
        match self {
            MarkdownNode::Headline(x) => x.stripped = stripped,
            MarkdownNode::ParagraphNode(x) => x.stripped = stripped,
            MarkdownNode::BlockStart(x) => x.stripped = stripped,
            MarkdownNode::BlockEnd(x) => x.stripped = stripped,
            MarkdownNode::LinkNode(x) => x.stripped = stripped,
        }
    }
    fn get_stripped(&self) -> Option<String> {
        match self {
            MarkdownNode::Headline(x) => x.stripped.clone(),
            MarkdownNode::ParagraphNode(x) => x.stripped.clone(),
            MarkdownNode::BlockStart(x) => x.stripped.clone(),
            MarkdownNode::BlockEnd(x) => x.stripped.clone(),
            MarkdownNode::LinkNode(x) => x.stripped.clone(),
        }
    }
}

pub(crate) fn parse_markdown(content: &str) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut pre: Vec<String> = Vec::new();
    let mut res = Vec::new();
    let lines = content.split("\n").collect_vec();
    for (idx, original_line) in lines.clone().into_iter().enumerate() {
        let mut stripped = String::new();
        let mut line = original_line.to_string();

        while !pre.is_empty() {
            if line
                .chars()
                .filter(|x| !x.is_whitespace())
                .collect::<String>()
                .starts_with(&pre.join(""))
            {
                let (cstripped, cline) = string::strip_prefix_with_whitespace(
                    &line,
                    &pre.clone().join("").chars().collect::<String>(),
                );
                line = cline;
                stripped = cstripped;

                break;
            } else {
                res.push(MarkdownNode::BlockEnd(BlockNode::new(
                    idx - 1,
                    pre.len(),
                    None,
                )));
                pre.pop();
            }
        }

        let mut line_stream = CharStream::new(&line.chars().collect_vec());

        let mut line_res = parse_line(&mut line_stream, idx, &mut pre)?;
        if line_res.is_empty() {
            // newline
            line_res.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                idx,
                "".to_owned(),
                None,
            )));
        }

        // Last line cleanup
        if idx + 1 == lines.len() {
            // close all blocks
            while let Some(_) = pre.iter().next() {
                line_res.push(MarkdownNode::BlockEnd(BlockNode::new(idx, pre.len(), None)));
                pre.pop();
            }
        }
        //TODO: add stripped to first element of line

        if !stripped.is_empty() {
            if let Some(x) = line_res.first_mut() {
                x.set_stripped(Some(stripped.clone()));
            } else {
                return Err(eyre!(format!(
                    "on line {}, expected to find at least one element, found none",
                    idx
                )));
            }
        }

        res.extend_from_slice(&line_res);
    }

    println!("res: {:?}", res);
    Ok(res)
}
fn parse_stream(
    line_stream: &mut CharStream<char>,
    index: usize,
    pre: &mut Vec<String>,
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
    line_stream: &mut CharStream<char>,
    index: usize,
    pre: &mut Vec<String>,
) -> color_eyre::Result<Vec<MarkdownNode>> {
    let mut res = Vec::new();

    res.extend_from_slice(&parse_stream(line_stream, index, pre)?);

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
                None,
            )));
            break;
        }
        let test = parse_stream(line_stream, index, pre)?;
        if !test.is_empty() || line_stream.is_empty() {
            // Paragraph stuff
            let p = current.clone().into_iter().collect::<String>();
            if !p.is_empty() {
                res.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                    index, p, None,
                )));
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
    for (idx, line) in lines {
        let mut res = String::new();
        for (_, l) in line.iter().enumerate() {
            if let Some(x) = l.get_stripped() {
                println!("[{}] got stripped {:?}", idx, x);
                res.push_str(&x);
            }
            res.push_str(&l.construct());
        }

        result.push(res);
    }
    Ok(result.join("\n"))
}
