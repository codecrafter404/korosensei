use color_eyre::eyre::OptionExt as _;

use crate::utils::char_stream::ItemStream;

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
    pub fn parse(content: &mut ItemStream<char>, line: usize) -> color_eyre::Result<Self>
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

    pub fn construct(&self) -> String {
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
