use itertools::Itertools as _;
use tokio::stream;

use crate::utils::char_stream::CharStream;

pub fn strip_prefix_with_whitespace(string: &str, prefix: &str) -> String {
    let mut res = vec![];
    let mut to_remove = prefix.chars().collect_vec();
    for char in string.chars() {
        if to_remove.is_empty() {
            res.push(char);
            continue;
        }
        if char.is_whitespace() {
            continue; // stip whitespace
        }
        if char == to_remove[0] {
            to_remove = to_remove[1..].to_vec();
            continue;
        }

        res.push(char);
    }
    res.into_iter().collect()
}

#[test]
fn test_strip_prefix_with_whitespace() {
    assert_eq!(
        strip_prefix_with_whitespace("a s d fhello world", "asdf"),
        "hello world".to_owned()
    );
    assert_eq!(
        strip_prefix_with_whitespace("a s d f hello world", "asdf"),
        " hello world".to_owned()
    );
    assert_eq!(
        strip_prefix_with_whitespace("as dfasdf", "asdf"),
        "asdf".to_owned()
    );
    assert_eq!(
        strip_prefix_with_whitespace("                content", ""),
        "                content".to_owned()
    );
}
