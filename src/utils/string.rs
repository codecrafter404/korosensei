use itertools::Itertools as _;

pub fn strip_prefix_with_whitespace(string: &str, prefix: &str) -> (String, String) {
    let mut res = vec![];
    let mut to_remove = prefix.chars().collect_vec();
    let mut stripped = Vec::new();
    for char in string.chars() {
        if to_remove.is_empty() {
            res.push(char);
            continue;
        }
        if char.is_whitespace() {
            stripped.push(char);
            continue; // stip whitespace
        }
        if char == to_remove[0] {
            stripped.push(char);
            to_remove = to_remove[1..].to_vec();
            continue;
        }

        res.push(char);
    }
    (stripped.into_iter().collect(), res.into_iter().collect())
}

#[test]
fn test_strip_prefix_with_whitespace() {
    assert_eq!(
        strip_prefix_with_whitespace("a s d fhello world", "asdf"),
        ("a s d f".to_owned(), "hello world".to_owned())
    );
    assert_eq!(
        strip_prefix_with_whitespace("a s d f hello world", "asdf"),
        ("a s d f".to_owned(), " hello world".to_owned())
    );
    assert_eq!(
        strip_prefix_with_whitespace("as dfasdf", "asdf"),
        ("as df".to_owned(), "asdf".to_owned())
    );
    assert_eq!(
        strip_prefix_with_whitespace("                content", ""),
        ("".to_owned(), "                content".to_owned())
    );
}
