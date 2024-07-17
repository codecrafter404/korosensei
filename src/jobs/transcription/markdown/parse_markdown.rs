use color_eyre::eyre::{eyre, OptionExt};
use itertools::Itertools;

use crate::utils::{char_stream::ItemStream, string};

use super::nodes::{
    block::BlockNode, headline::HeadlineNode, html::HtmlNode, link::LinkNode,
    paragraph::ParagraphNode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkdownNode {
    Headline(HeadlineNode),
    ParagraphNode(ParagraphNode),
    BlockStart(BlockNode),
    BlockEnd(BlockNode),
    LinkNode(LinkNode),
    HtmlNode(HtmlNode),
}
impl MarkdownNode {
    fn get_line(&self) -> usize {
        match &self {
            MarkdownNode::Headline(x) => x.line,
            MarkdownNode::ParagraphNode(x) => x.line,
            MarkdownNode::BlockStart(x) => x.line,
            MarkdownNode::BlockEnd(x) => x.line,
            MarkdownNode::LinkNode(x) => x.line,
            MarkdownNode::HtmlNode(x) => x.line,
        }
    }
    fn construct(&self) -> String {
        match &self {
            MarkdownNode::Headline(x) => x.construct(),
            MarkdownNode::ParagraphNode(x) => x.construct(),
            MarkdownNode::BlockStart(x) => x.construct(),
            MarkdownNode::BlockEnd(_) => String::new(),
            MarkdownNode::LinkNode(x) => x.construct(),
            MarkdownNode::HtmlNode(x) => x.construct(),
        }
    }
    fn set_stripped(&mut self, stripped: Option<String>) {
        match self {
            MarkdownNode::Headline(x) => x.stripped = stripped,
            MarkdownNode::ParagraphNode(x) => x.stripped = stripped,
            MarkdownNode::BlockStart(x) => x.stripped = stripped,
            MarkdownNode::BlockEnd(x) => x.stripped = stripped,
            MarkdownNode::LinkNode(x) => x.stripped = stripped,
            MarkdownNode::HtmlNode(x) => x.stripped = stripped,
        }
    }
    fn get_stripped(&self) -> Option<String> {
        match self {
            MarkdownNode::Headline(x) => x.stripped.clone(),
            MarkdownNode::ParagraphNode(x) => x.stripped.clone(),
            MarkdownNode::BlockStart(x) => x.stripped.clone(),
            MarkdownNode::BlockEnd(x) => x.stripped.clone(),
            MarkdownNode::LinkNode(x) => x.stripped.clone(),
            MarkdownNode::HtmlNode(x) => x.stripped.clone(),
        }
    }
    pub fn get_html(&self) -> Option<HtmlNode> {
        match self.clone() {
            MarkdownNode::HtmlNode(x) => Some(x),
            _ => None,
        }
    }
    pub fn get_headline(&self) -> Option<HeadlineNode> {
        match self.clone() {
            MarkdownNode::Headline(x) => Some(x),
            _ => None,
        }
    }
    pub fn get_paragraph(&self) -> Option<ParagraphNode> {
        match self.clone() {
            MarkdownNode::ParagraphNode(x) => Some(x),
            _ => None,
        }
    }
    pub fn get_block_start(&self) -> Option<BlockNode> {
        match self.clone() {
            MarkdownNode::BlockStart(x) => Some(x),
            _ => None,
        }
    }
    pub fn get_block_end(&self) -> Option<BlockNode> {
        match self.clone() {
            MarkdownNode::BlockEnd(x) => Some(x),
            _ => None,
        }
    }
    pub fn get_link(&self) -> Option<LinkNode> {
        match self.clone() {
            MarkdownNode::LinkNode(x) => Some(x),
            _ => None,
        }
    }
    pub fn increment_line_by(&mut self, offset: usize) {
        match self {
            MarkdownNode::Headline(x) => x.line += offset,
            MarkdownNode::ParagraphNode(x) => x.line += offset,
            MarkdownNode::BlockStart(x) => x.line += offset,
            MarkdownNode::BlockEnd(x) => x.line += offset,
            MarkdownNode::LinkNode(x) => x.line += offset,
            MarkdownNode::HtmlNode(x) => x.line += offset,
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

        let mut line_stream = ItemStream::new(&line.chars().collect_vec());

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

    Ok(res)
}
fn parse_stream(
    line_stream: &mut ItemStream<char>,
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
    if line_stream.test(|x| x == '<').is_some_and(|x| x) {
        if let Some(x) = HtmlNode::parse(line_stream, index)? {
            res.push(MarkdownNode::HtmlNode(x));
        }
    }

    return Ok(res);
}
fn parse_line(
    line_stream: &mut ItemStream<char>,
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
                res.push_str(&x);
            }
            res.push_str(&l.construct());
        }

        result.push(res);
    }
    Ok(result.join("\n"))
}
