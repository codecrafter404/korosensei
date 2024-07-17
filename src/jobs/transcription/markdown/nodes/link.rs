use color_eyre::eyre::eyre;

use crate::utils::char_stream::ItemStream;

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
    pub fn parse(
        stream: &mut ItemStream<char>,
        line: usize,
    ) -> color_eyre::Result<Option<LinkNode>> {
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
