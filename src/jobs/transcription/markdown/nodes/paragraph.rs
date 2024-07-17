use crate::utils::char_stream::ItemStream;

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

    /// Gets the leading whitespace
    pub fn get_whitespace(&self) -> String {
        let mut stream = ItemStream::new(&self.content.chars().collect());

        stream
            .take_while(|x| x.is_whitespace())
            .into_iter()
            .collect()
    }
}
