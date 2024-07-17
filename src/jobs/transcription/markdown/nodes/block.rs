use color_eyre::eyre::eyre;

use crate::utils::char_stream::ItemStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockNode {
    /// at this line is the first or last '>'
    pub line: usize,
    /// nested level
    pub level: usize,
    pub stripped: Option<String>,
}
impl BlockNode {
    pub fn parse(
        content: &mut ItemStream<char>,
        line: usize,
        level: usize,
    ) -> color_eyre::Result<BlockNode> {
        if content.take(1) != vec!['>'] {
            return Err(eyre!("Expected to get Block starting with '>'"));
        }
        Ok(BlockNode::new(line, level, None))
    }

    pub fn construct(&self) -> String {
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
