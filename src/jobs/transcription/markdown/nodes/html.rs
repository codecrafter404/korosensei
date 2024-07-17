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
        println!("------- [{}]", line);
        let bak = stream.clone();
        if stream.take(1) != vec!['<'] {
            return Err(eyre!("Expected to get link starting with '['"));
        }
        let mut original = Vec::new();
        original.push('<');

        let comment_indicator = stream.take(1);

        if comment_indicator.is_empty() {
            // only '<' EOL
            *stream = bak;
            return Ok(None);
        }

        original.extend_from_slice(&comment_indicator);
        let mut tag = String::new();
        if comment_indicator == vec!['!'] {
            println!("-> comment");
            // comment

            tag = "<!--...-->".into();
            let arrow_line = stream.take(2);
            if arrow_line != vec!['-', '-'] {
                *stream = bak;
                return Ok(None);
            }
            original.extend_from_slice(&arrow_line);

            let mut content = Vec::new();

            loop {
                match stream.test_window(vec!['-', '-', '>']) {
                    Some(x) => {
                        if !x {
                            let c = stream.take(1);
                            content.extend_from_slice(&c);
                            original.extend_from_slice(&c);
                            continue;
                        } else {
                            original.extend_from_slice(&stream.take(3));
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
                original.into_iter().join(""),
                "".to_string(),
                None,
            )));
        } else {
            println!("-> normal tag");
            // normal tag
            tag.push_str(&comment_indicator[0].to_string());
            let ttag = stream.take_while(|x| !vec![' ', '/', '>'].contains(&x));
            original.extend_from_slice(&ttag);
            tag.push_str(&ttag.into_iter().join(""));
            let mut attribute = String::new();
            match stream.test(|x| x == ' ') {
                Some(x) => {
                    if x {
                        println!("-> attribute");
                        original.extend_from_slice(&stream.take(1));
                        let attr = stream.take_while(|x| !vec!['/', '>'].contains(&x));
                        original.extend_from_slice(&attr);
                        if !attr.is_empty() {
                            attribute = attr.into_iter().join("");
                        }
                    }
                }
                None => {
                    println!("-> attr EOL");
                    // EOL
                    *stream = bak;
                    return Ok(None);
                }
            }

            // handle self closing tags
            match stream.test_window(vec!['/', '>']) {
                Some(x) => {
                    if x {
                        println!("-> self closing");
                        original.extend_from_slice(&stream.take(2));
                        return Ok(Some(HtmlNode::new(
                            line,
                            tag,
                            "".into(),
                            original.into_iter().join(""),
                            attribute,
                            None,
                        )));
                    }
                }
                None => {
                    println!("-> self closing EOL {:?}", stream.preview(stream.len()));
                    // EOL
                    *stream = bak;
                    return Ok(None);
                }
            }
            match stream.test(|x| x == '>') {
                Some(x) => {
                    if !x {
                        // then '/>' test failed
                        return Err(eyre!(format!(
                            "Expected to get a closing tag on line {}",
                            line
                        )));
                    }
                    original.extend_from_slice(&stream.take(1));
                    println!("-> normal closing");
                }
                None => {
                    println!("-> normal closing EOL");
                    // EOL
                    *stream = bak;
                    return Ok(None);
                }
            }

            let window = format!("</{}>", tag).chars().collect_vec();
            let mut content = Vec::new();
            println!("-> window search: {:?}", content);
            loop {
                match stream.test_window_custom(window.clone(), |(a, b)| {
                    a.to_ascii_lowercase() == b.to_ascii_lowercase()
                }) {
                    Some(x) => {
                        if x {
                            original.extend_from_slice(&stream.take(window.len()));
                            return Ok(Some(HtmlNode::new(
                                line,
                                tag,
                                content.into_iter().collect(),
                                original.into_iter().collect(),
                                attribute,
                                None,
                            )));
                        } else {
                            let c = stream.take(1);
                            content.extend_from_slice(&c);
                            original.extend_from_slice(&c);
                        }
                    }
                    None => {
                        // EOL
                        *stream = bak;
                        return Ok(None);
                    }
                }
            }
        }
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
