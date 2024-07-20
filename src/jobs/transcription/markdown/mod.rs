use std::ops::Sub as _;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use color_eyre::eyre::{eyre, ContextCompat, OptionExt};
use nodes::block::BlockNode;
use nodes::link::LinkNode;
use nodes::paragraph::ParagraphNode;
use parse_markdown::MarkdownNode;
use serde::de::Expected;

use crate::utils::char_stream::ItemStream;
use crate::utils::config::Config;
use crate::utils::git::{self};
use itertools::Itertools;

mod nodes;
mod parse_markdown;
mod test_data;
mod test_markdown_parse;

#[derive(Debug, Clone)]
pub(crate) struct CorrelatingFile {
    /// Path to .md file
    pub path: PathBuf,
    /// Headlines index, starting by 0
    pub headlines: Vec<u64>,
    /// content
    pub content: String,
}
impl CorrelatingFile {
    pub(crate) fn link_to_transcript(
        &self,
        transcript_path: PathBuf,
        content: &str,
        transcript_time: &DateTime<Utc>,
    ) -> color_eyre::Result<String> {
        let parsed = parse_markdown::parse_markdown(content)?;
        println!("got parsed: {:#?}", parsed);
        let mut stream = ItemStream::new(&parsed.into_iter().rev().collect_vec());
        let whitespace = stream.take_while(|x| {
            x.get_paragraph()
                .is_some_and(|x| x.content.trim().is_empty())
        });
        let mut stream = ItemStream::new(&stream.collect().into_iter().rev().collect_vec());

        let mut result_buf = Vec::new();

        let mut headlines = self
            .headlines
            .clone()
            .into_iter()
            .map(|x| x as usize)
            .collect_vec();
        headlines.sort();
        headlines.dedup();

        let mut general_offset = 0;
        for headline in headlines {
            let mut line_offset = 0;
            println!("-> searchline_offset: {}", general_offset);
            result_buf.extend_from_slice(&stream.take_while(|x| {
                !x.get_headline()
                    .is_some_and(|x| x.line == (headline + general_offset))
            }));
            let h = stream
                .take_one()
                .wrap_err(eyre!(format!(
                    "expected to get headline on line {}; got nothing",
                    headline
                )))?
                .get_headline()
                .ok_or_eyre(eyre!(format!(
                    "expected to get headline on line {}",
                    headline
                )))?;
            println!("-> H1: {:?}", h);
            result_buf.push(MarkdownNode::Headline(h.clone()));
            let htmls = stream.take_while(|x| {
                x.get_html().is_some() || x.get_paragraph().is_some_and(|x| x.content == "")
            });
            println!("-> HTMLs: {:?}", htmls);
            result_buf.extend_from_slice(&htmls);
            // check if is block
            // true: skip past all empty paragraphs -> check if paragraph =

            let mut need_header = true;
            let mut block_end = None;
            let bak = stream.clone();
            let mut block_nodes = vec![];
            loop {
                let next = stream.take_one();
                match next {
                    Some(x) => {
                        if let Some(block_start) = x.get_block_start() {
                            println!("-> [{}] Existing block", result_buf.len());
                            // existing block
                            println!("-> prev: {:?}", stream.preview(1));
                            block_nodes.push(x.clone());
                            block_nodes.extend_from_slice(&stream.take_while(|x| {
                                x.get_paragraph()
                                    .is_some_and(|x| x.content == "" || x.content == " ")
                            }));
                            if stream
                                .test(|x| {
                                    x.get_paragraph()
                                        .is_some_and(|x| x.content.trim() == "_Links")
                                })
                                .is_some_and(|x| x)
                            {
                                block_nodes.extend_from_slice(&stream.take(1));

                                println!("-> prev: {:?}", stream.preview(1));
                                println!("-> [{}] all empty lines before links", result_buf.len());
                                let mut empty_line_counter = block_nodes.len();
                                // all empty lines before links
                                block_nodes.extend_from_slice(&stream.take_while(|x| {
                                    x.get_paragraph()
                                        .is_some_and(|x| x.content == "" || x.content == "")
                                }));
                                empty_line_counter = block_nodes.len() - empty_line_counter;

                                println!("-> prev: {:?}", stream.preview(1));
                                println!("-> [{}] existing links", result_buf.len());
                                // existing links
                                let mut _break = false;
                                loop {
                                    let block_nodes_bak = block_nodes.clone();
                                    let mut line = None;
                                    if stream
                                        .test(|x| {
                                            x.get_paragraph()
                                                .is_some_and(|x| x.content.trim().is_empty())
                                        })
                                        .is_some_and(|x| x)
                                    {
                                        let c =
                                            stream.take(1)[0].get_paragraph().expect("Infallible");
                                        line = Some(c.line.clone());
                                        block_nodes.push(MarkdownNode::ParagraphNode(c));
                                        println!("-> prev: {:?}", stream.preview(1));
                                    }
                                    if stream.test(|x| x.get_link().is_some()).is_some_and(|x| x) {
                                        let c = stream
                                            .take_one()
                                            .expect("Infallible")
                                            .get_link()
                                            .expect("Infallible");
                                        if let Some(line) = line {
                                            if line != c.line {
                                                // it was a newline
                                                block_nodes = block_nodes_bak;
                                                break;
                                            }
                                        }
                                        block_nodes.push(MarkdownNode::LinkNode(c));
                                        if empty_line_counter == 0 {
                                            _break = true;
                                            // no empty lines :(
                                            break;
                                        }
                                    } else {
                                        if line.is_none() {
                                            // give up
                                            break;
                                        }
                                    }
                                }

                                if _break {
                                    println!("-> we dont have at least one empty line between _Links & [link]()");
                                    stream = bak;
                                    break;
                                }

                                println!("-> prev: {:?}", stream.preview(1));

                                println!("-> [{}] expected end of block", result_buf.len());
                                // expected end of block
                                if stream
                                    .test(|x| {
                                        x.get_block_end()
                                            .is_some_and(|x| x.level == block_start.level)
                                    })
                                    .is_some_and(|x| x)
                                {
                                    block_end = Some(
                                        stream
                                            .take_one()
                                            .expect("Infallible")
                                            .get_block_end()
                                            .expect("Infallible"),
                                    );
                                    result_buf.extend_from_slice(&block_nodes);

                                    println!("-> prev: {:?}", stream.preview(1)); // DEBUG: block end should be on line
                                    need_header = false;
                                    break;
                                } else {
                                    println!("-> no eob");
                                    stream = bak;
                                    break;
                                }
                            } else {
                                println!("-> prev: {:?}", stream.preview(1));
                                println!("-> no header");
                                stream = bak;
                                println!("-> prev: {:?}", stream.preview(1));
                            }
                            break;
                        } else {
                            println!("-> next is not block start, its {:?}", x);

                            if stream
                                .test(|y| {
                                    y.get_block_start().is_some() && y.get_line() == x.get_line()
                                })
                                .is_some_and(|x| x)
                                && x.get_paragraph()
                                    .is_some_and(|x| x.content.trim().is_empty())
                            {
                                println!("-> searching forward {:?}", stream.preview(1));
                                block_nodes.push(x);
                                continue;
                            } else {
                                stream = bak;
                                break;
                            }
                        }
                    }
                    None => {
                        println!("-> no existing block");
                        stream = bak;
                        break;
                    }
                }
            }
            println!(
                "-> [{}] need_header: {}, last_block: {:?}",
                result_buf.len(),
                need_header,
                block_end
            );
            let last_item = result_buf
                .clone()
                .into_iter()
                .chunk_by(|x| x.get_line())
                .into_iter()
                .map(|(a, b)| (a, b.collect_vec()))
                .collect_vec()
                .last()
                .ok_or_eyre("Expected to have at least one item")?
                .1
                .first()
                .expect("Infallible")
                .clone();
            let last_block_level = result_buf
                .iter()
                .rev()
                .find(|x| {
                    (x.get_block_start().is_some() || x.get_block_end().is_some())
                        && x.get_line() >= h.line
                })
                .map(|x| match x {
                    MarkdownNode::BlockStart(x) => x.level,
                    MarkdownNode::BlockEnd(x) => x.level - 1,
                    _ => 0,
                })
                .unwrap_or(0);
            let mut stripped = last_item.get_stripped();
            println!(
                "-> [{}] last_item: {:?}; last_block_level: {:?}; prev: {:?}",
                result_buf.len(),
                last_item,
                last_block_level,
                stream.preview(3)
            );
            let mut new_line_offset = 0;
            if need_header {
                // add spacing if needed

                let need_spacing_p = last_item
                    .get_paragraph()
                    .is_some_and(|x| !x.get_whitespace().is_empty());
                println!("-> need spacing?: {}", need_spacing_p);
                if need_spacing_p {
                    // we need to add an paragraph header
                    result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                        last_item.get_line() + 1,
                        last_item
                            .get_paragraph()
                            .ok_or_eyre("Infallible")?
                            .get_whitespace(),
                        stripped.clone(),
                    )));
                }

                println!("-> [{}] pushing header", result_buf.len());
                result_buf.push(MarkdownNode::BlockStart(BlockNode::new(
                    last_item.get_line() + 1,
                    last_block_level,
                    if !need_spacing_p {
                        stripped.clone()
                    } else {
                        None
                    },
                )));
                result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                    last_item.get_line() + 1,
                    " _Links".into(),
                    None,
                )));

                line_offset += 1;

                result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                    last_item.get_line() + 2,
                    " ".into(),
                    Some(format!(
                        "{}{}>",
                        last_item.get_stripped().unwrap_or_default(),
                        if need_spacing_p {
                            last_item
                                .get_paragraph()
                                .ok_or_eyre("Infallible")?
                                .get_whitespace()
                        } else {
                            String::new()
                        }
                    )),
                )));
                line_offset += 1;
                println!("-> pushing link seperation section");
                result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                    last_item.get_line() + 3,
                    " ".into(),
                    Some(format!(
                        "{}{}>",
                        last_item.get_stripped().unwrap_or_default(),
                        if need_spacing_p {
                            last_item
                                .get_paragraph()
                                .ok_or_eyre("Infallible")?
                                .get_whitespace()
                        } else {
                            String::new()
                        }
                    )),
                )));
                line_offset += 1;
            } else {
                println!("-> dont need header");
                let line = result_buf
                    .clone()
                    .into_iter()
                    .chunk_by(|x| x.get_line())
                    .into_iter()
                    .map(|(a, b)| (a, b.collect_vec()))
                    .collect_vec()
                    .last()
                    .ok_or_eyre("Expect to have at least one last item in result_buf")?
                    .1
                    .clone();

                let whitespace = match line.last().expect("Infallible") {
                    MarkdownNode::ParagraphNode(x) => {
                        println!("-> we got empty line / _Links");
                        if x.content.trim() == "_Links" {
                            println!("-> _Links");
                            // get whitespace from there
                            if let Some(y) = stripped {
                                stripped = Some(format!("{}>", y));
                            } else if !x.get_whitespace().is_empty() {
                                stripped = Some(format!(">"));
                            }

                            println!("-> pushing link seperation section");
                            result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                                last_item.get_line() + 1,
                                x.get_whitespace(),
                                stripped.clone(),
                            )));
                            line_offset += 1;
                            new_line_offset += 1;

                            x.get_whitespace()
                        } else if x.content.trim().is_empty() {
                            println!("-> newline");
                            // is empty line
                            x.content.clone()
                        } else {
                            return Err(eyre!(format!(
                                "content should be whitespace or _Links, got: {:?}",
                                x.content
                            )));
                        }
                    }
                    MarkdownNode::LinkNode(_) => {
                        println!("-> got link node; searching for paragraph to find whitespace");
                        match line.iter().find(|x| {
                            x.get_paragraph()
                                .is_some_and(|x| x.content.trim().is_empty())
                        }) {
                            Some(x) => {
                                println!("-> Found p {:?}", x);
                                // take whitespace from here
                                let p = x.get_paragraph().expect("Infallible");
                                p.get_whitespace()
                            }
                            None => {
                                println!("-> found none: line: {:?}", line);
                                String::new()
                            }
                        }
                    }
                    x => {
                        return Err(eyre!(format!(
                            "Expected to get paragraph / link node, got: {:?}",
                            x
                        )))
                    }
                };

                println!(
                    "-> got whitespace {:?}; stripped: {:?}; adding space paragraph",
                    whitespace, stripped
                );
                if !whitespace.is_empty() {
                    result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                        last_item.get_line() + 1 + new_line_offset,
                        whitespace,
                        stripped.clone(),
                    )));
                    line_offset += 1;
                } else {
                    println!("-> not added paragraph; whitespace is empty");
                }
            }
            let last_line_items = result_buf
                .clone()
                .into_iter()
                .chunk_by(|x| x.get_line())
                .into_iter()
                .map(|(a, b)| (a, b.collect_vec()))
                .collect_vec()
                .last()
                .ok_or_eyre("Expected to have at least one item")?
                .1
                .clone();
            let last_item = last_line_items.first().expect("Infallible").clone();

            println!(
                "-> [{}] pushing link; last_line_items: {:?}",
                result_buf.len(),
                last_line_items
            );
            result_buf.push(MarkdownNode::LinkNode(LinkNode::new(
                last_item.get_line(),
                transcript_time.format("%d.%m.%Y %H:%M").to_string(),
                url_escape::encode_path(
                    &transcript_path
                        .to_str()
                        .ok_or_eyre("expected transcription path to be parsable")?
                        .to_string(),
                )
                .to_string(),
                if last_line_items.is_empty() {
                    stripped
                } else {
                    None
                },
            )));
            // line_offset += 1;

            if need_header {
                println!("-> [{}] pushing end block", result_buf.len());
                let last_block_level = result_buf
                    .iter()
                    .rev()
                    .find(|x| x.get_block_start().is_some() || x.get_block_end().is_some())
                    .map(|x| match x {
                        MarkdownNode::BlockStart(x) => x.level,
                        MarkdownNode::BlockEnd(x) => x.level - 1,
                        _ => 0,
                    })
                    .unwrap_or(0);
                result_buf.push(MarkdownNode::BlockEnd(BlockNode::new(
                    last_item.get_line(),
                    last_block_level,
                    None,
                )));
            } else if let Some(x) = block_end {
                println!("-> [{}] pushing endblock (existing)", result_buf.len());
                let mut x = MarkdownNode::BlockEnd(x);
                x.increment_line_by(line_offset);
                result_buf.push(x);
            } else {
                return Err(eyre!("Infallible: this should never happen :("));
            }

            let last_line_before_block = result_buf
                .clone()
                .into_iter()
                .rev()
                .skip_while(|x| !x.get_block_start().is_some())
                .skip(1)
                .collect_vec()
                .into_iter()
                .rev()
                .chunk_by(|x| x.get_line())
                .into_iter()
                .map(|(a, b)| (a, b.collect_vec()))
                .collect_vec()
                .last()
                .ok_or_eyre("Expected to have at least one item")?
                .1
                .clone();

            println!("-> newline_offset: {}", new_line_offset);
            // adding an empty line after
            if stream
                .test(|x| {
                    x.get_paragraph().is_some_and(|x| {
                        x.content.trim().is_empty()
                            && (x.line + new_line_offset) == last_item.get_line()
                    })
                })
                .is_some_and(|x| !x)
            {
                println!("-> adding empty paragraph for seperation");
                println!(
                    "-> last_item: {:?}; prev: {:?}",
                    last_item,
                    stream.preview(1)
                );
                result_buf.push(MarkdownNode::ParagraphNode(ParagraphNode::new(
                    last_item.get_line() + 1,
                    "".into(),
                    last_line_before_block
                        .first()
                        .ok_or_eyre("Infallible")?
                        .get_stripped(),
                )));
                line_offset += 1;
            } else {
                println!(
                    "-> already got empty line: {:?}; last_line_before_block: {:?}, last_line: {:?}, elem_at_skip {:?}",
                    stream.preview(1),
                    last_line_before_block,
                    last_line_items,
                    result_buf.iter().rev().skip(line_offset).next()
                );
            }

            println!(
                "-> [{}] incrementing lines by offset {}",
                result_buf.len(),
                line_offset
            );
            println!("-> prev: {:?}", stream.preview(1));
            // Increments all the following nodes
            stream = ItemStream::new(
                &stream
                    .collect()
                    .into_iter()
                    .map(|x| {
                        let mut x = x;
                        x.increment_line_by(line_offset);
                        x
                    })
                    .collect_vec(),
            );
            println!("-> ---------------- End: [{}]", result_buf.len());
            general_offset += line_offset;
        }
        if !stream.is_empty() {
            result_buf.extend_from_slice(&stream.collect());
        }
        let whitespace = whitespace
            .into_iter()
            .map(|x| {
                let mut x = x;
                x.increment_line_by(general_offset);
                x
            })
            .collect_vec();
        result_buf.extend_from_slice(&whitespace);
        println!("{:#?}", result_buf);
        let res = parse_markdown::construct_markdown(result_buf)?;

        Ok(res)
    }
}
#[test]
fn test_eof_1() {
    let file = CorrelatingFile {
        path: PathBuf::new(),
        headlines: vec![0],
        content: String::new(),
    };
    let input_content = "\
# Hello World
";
    let expected = "\
# Hello World
> _Links
> 
> [14.07.2024 12:00](hello)
";

    let actual_result = file
        .link_to_transcript(
            PathBuf::from_str("hello").unwrap(),
            input_content,
            &DateTime::from_timestamp(1720958400, 0).unwrap(),
        )
        .unwrap();
    println!("{:#?}", actual_result);
    let actual_result = actual_result.split("\n").collect_vec();
    let expected = expected.split("\n").collect_vec();
    for (idx, e) in expected.into_iter().enumerate() {
        assert_eq!(actual_result[idx], e, "[{}]", idx);
    }
}

// #[test]
// fn test_eof_2() {
//     let file = CorrelatingFile {
//         path: PathBuf::new(),
//         headlines: vec![0],
//         content: String::new(),
//     };
//     let input_content = "\
// # Hello World
//
// > _Links
// >
// > []()
// ";
//     let expected = "\
// # Hello World
//
// > _Links
// >
// > []()
// > [14.07.2024 12:00](hello)
// ";
//
//     let actual_result = file
//         .link_to_transcript(
//             PathBuf::from_str("hello").unwrap(),
//             input_content,
//             &DateTime::from_timestamp(1720958400, 0).unwrap(),
//         )
//         .unwrap();
//     println!("{:#?}", actual_result);
//     let actual_result = actual_result.split("\n").collect_vec();
//     let expected = expected.split("\n").collect_vec();
//     for (idx, e) in expected.into_iter().enumerate() {
//         assert_eq!(actual_result[idx], e, "[{}]", idx);
//     }
// }
#[test]
fn test_whitespace_in_link() {
    let file = CorrelatingFile {
        path: PathBuf::new(),
        headlines: vec![0],
        content: String::new(),
    };
    let input_content = "\
# Hello World";
    let expected = "\
# Hello World
> _Links
> 
> [14.07.2024 12:00](hello%20world)";

    let actual_result = file
        .link_to_transcript(
            PathBuf::from_str("hello world").unwrap(),
            input_content,
            &DateTime::from_timestamp(1720958400, 0).unwrap(),
        )
        .unwrap();
    println!("{:#?}", actual_result);
    let actual_result = actual_result.split("\n").collect_vec();
    let expected = expected.split("\n").collect_vec();
    for (idx, e) in expected.into_iter().enumerate() {
        assert_eq!(actual_result[idx], e, "[{}]", idx);
    }
}

#[test]
fn test_corelating_file_linkage_full() {
    let file = CorrelatingFile {
        path: PathBuf::new(),
        headlines: vec![0, 4, 17, 21, 25, 28, 31, 35],
        content: String::new(),
    };

    let input_content = "\
# Hello world
<!-- test comment -->
> Normal callout
content
### Append Test
>
>
>
> _Links
>
> [Example]()
> []()

> callout
> _Links
> those are great
content
##### Append Test #2
> _Links


##### Append Test #3
> _Links
> []()
> broken
## Hello world
        content
> content?
> # This is also a heading
> > test
> content
> ## Subheading
> > _Links
> >
> > [Existing_link](https://asdf.com)
# Not working
> _Links
> []()";
    let expected = "\
# Hello world
<!-- test comment -->
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

> Normal callout
content
### Append Test
>
>
>
> _Links
>
> [Example]()
> []()
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

> callout
> _Links
> those are great
content
##### Append Test #2
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)


##### Append Test #3
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

> _Links
> []()
> broken
## Hello world
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

        content
> content?
> # This is also a heading
> > _Links
> > 
> > [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)
>
> > test
> content
> ## Subheading
> > _Links
> >
> > [Existing_link](https://asdf.com)
> > [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)
>
# Not working
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

> _Links
> []()";
    let actual_result = file
        .link_to_transcript(
            PathBuf::from_str("/assets/transcriptions/asdf.transcript.md").unwrap(),
            input_content,
            &DateTime::from_timestamp(1720958400, 0).unwrap(),
        )
        .unwrap();
    println!("{:#?}", actual_result);
    let actual_result = actual_result.split("\n").collect_vec();
    let expected = expected.split("\n").collect_vec();
    for (idx, e) in expected.into_iter().enumerate() {
        assert_eq!(actual_result[idx], e, "[{}]", idx);
    }
}

#[test]
fn harder_tests() {
    let file = CorrelatingFile {
        path: PathBuf::new(),
        headlines: vec![0, 2],
        content: String::new(),
    };
    let input_content = "\
# Hello world
> > _Links
# Hello second world
>>_Links
>>
>>[]()";
    let expected = "\
# Hello world
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

> > _Links
# Hello second world
> _Links
> 
> [14.07.2024 12:00](/assets/transcriptions/asdf.transcript.md)

>>_Links
>>
>>[]()";
    let actual_result = file
        .link_to_transcript(
            PathBuf::from_str("/assets/transcriptions/asdf.transcript.md").unwrap(),
            input_content,
            &DateTime::from_timestamp(1720958400, 0).unwrap(),
        )
        .unwrap();
    println!("{:#?}", actual_result);
    let actual_result = actual_result.split("\n").collect_vec();
    let expected = expected.split("\n").collect_vec();
    for (idx, e) in expected.into_iter().enumerate() {
        assert_eq!(actual_result[idx], e, "[{}]", idx);
    }
}

/// gets the nearest (direction: up) heading
/// when `include_parents == true` then also the next parent headings
pub(crate) fn get_related_markdown_headings(
    line: u64,
    content: &str,
    include_parents: bool,
) -> color_eyre::Result<Vec<u64>> {
    let lines = content.split("\n").collect::<Vec<_>>();
    if line as usize >= lines.len() {
        return Err(eyre!("searchline out of index"));
    }
    let mut lines = lines
        .into_iter()
        .take((line + 1) as usize)
        .collect::<Vec<_>>();
    lines.reverse();
    println!("{:?}", lines);

    let mut my_level = usize::MAX;
    let mut res = vec![];
    for (idx, line_str) in lines.into_iter().enumerate() {
        if let Some((_, level, _)) = lazy_regex::regex_captures!("^[\\s>]*(#{1,})(.*)$", line_str) {
            println!("{} [{}]", level, my_level);
            let level = level.len();
            if my_level > level {
                my_level = level;
                res.push(line - idx as u64);
            }
            if !include_parents {
                break;
            } else if my_level <= 1 {
                break;
            }
        }
    }

    return Ok(res);
}

#[test]
fn test_markdown_heading_parser() {
    let input = "content
# 1.0 Heading
content
    ## 1.2 Heading
content
## 1.3 Heading
content
>
>   # Heading
>   content
>   ## second heading
>   content
";
    let res_pattern = [
        (0, false, vec![]),
        (1, true, vec![1]),
        (4, false, vec![3]),
        (4, true, vec![3, 1]),
        (8, true, vec![8]),
        (11, true, vec![10, 8]),
    ];
    for (line, parent, res) in res_pattern {
        assert_eq!(
            get_related_markdown_headings(line, &input, parent).unwrap(),
            res,
            "Parsing line {}",
            line
        );
    }
}
