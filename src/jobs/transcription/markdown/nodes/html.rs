use color_eyre::eyre::eyre;
use itertools::Itertools;

use crate::utils::char_stream::ItemStream;

//NOTE: this is a simple implementation: the content will not be parsed, and multiline html tags cant be parsed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlNode {
    pub line: usize,
    pub tag: String,
    pub content: String,
    pub attribute: String,
    pub original: String,

    pub stripped: Option<String>,
}
impl HtmlNode {
    pub fn parse(
        stream: &mut ItemStream<char>,
        line: usize,
    ) -> color_eyre::Result<Option<HtmlNode>> {
        let bak = stream.clone();
        if stream.take(1) != vec!['<'] {
            return Err(eyre!("Expected to get link starting with '['"));
        }
        let mut original = String::new();
        original.push_str("<");

        let comment_indicator = stream.take(1);

        if comment_indicator.is_empty() {
            // only '<' EOL
            *stream = bak;
            return Ok(None);
        }

        let mut tag = String::new();
        if comment_indicator == vec!['!'] {
            // comment

            tag = "<!--...-->".into();
            let arrow_line = stream.take(2);
            if arrow_line != vec!['-', '-'] {
                *stream = bak;
                return Ok(None);
            }
            original.push_str(&arrow_line.iter().join(""));

            let mut content = Vec::new();

            loop {
                match stream.test_window(vec!['-', '-', '>']) {
                    Some(x) => {
                        if !x {
                            let c = stream.take(1);
                            content.extend_from_slice(&c);
                            original.push_str(&c.iter().join(""));
                            continue;
                        } else {
                            original.push_str(&stream.take(3).into_iter().join(""));
                            break;
                        }
                    }
                    None => {
                        *stream = bak;
                        return Ok(None);
                    }
                }
            }
            return Ok(Some(HtmlNode::new(
                line,
                tag,
                content.into_iter().join(""),
                original,
                "".to_string(),
                None,
            )));
        } else {
            // normal tag
            tag.push_str(&comment_indicator[0].to_string());
            tag.push_str(
                &stream
                    .take_while(|x| !vec![' ', '/', '>'].contains(&x))
                    .into_iter()
                    .join(""),
            );
            match stream.test(|x| x == ' ') {
                Some(x) => {}
                None => {
                    // EOL
                    *stream = bak;
                    return Ok(None);
                }
            }
        }

        unimplemented!()
    }
    pub fn new(
        line: usize,
        tag: String,
        content: String,
        original: String,
        attribute: String,
        stripped: Option<String>,
    ) -> HtmlNode {
        HtmlNode {
            line,
            tag,
            content,
            stripped,
            original,
            attribute,
        }
    }
    pub fn construct(&self) -> String {
        self.original.clone()
    }
}
