use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharStream {
    chars: Vec<char>,
}
impl CharStream {
    pub fn take(&mut self, n: usize) -> Vec<char> {
        let mut res = Vec::new();
        for _ in 0..n {
            if let Some(x) = self.chars.pop() {
                res.push(x);
            }
        }
        res
    }
    fn preview(&self, n: usize) -> Vec<char> {
        let n = n.min(self.chars.len() - 1);
        let mut res = Vec::new();
        for i in 0..n {
            res.push(self.chars[i].clone());
        }
        res
    }
    pub fn len(&self) -> usize {
        return self.chars.len();
    }
    pub fn test_n<F>(&self, n: usize, test_function: F) -> Vec<bool>
    where
        F: Fn(char) -> bool,
    {
        let to_test = self.preview(n);
        to_test.into_iter().map(|x| test_function(x)).collect_vec()
    }
    pub fn test<F>(&self, test_function: F) -> Option<bool>
    where
        F: Fn(char) -> bool,
    {
        self.test_n(1, test_function).into_iter().next()
    }
    pub fn test_while<F>(&self, test_function: F) -> usize
    where
        F: Fn(char) -> bool,
    {
        let mut current: usize = 0;
        for x in self.chars.iter() {
            current += 1;
            if !test_function(*x) {
                break;
            }
        }
        current
    }
    pub fn new(chars: &Vec<char>) -> Self {
        let mut chars = chars.clone();
        chars.reverse();
        CharStream { chars }
    }
    pub fn collect(&mut self) -> Vec<char> {
        self.take(self.chars.len())
    }
}

#[test]
fn test_char_stream() {
    let mut stream = CharStream::new(&vec!['a', 'b', 'c', 'd', 'e', 'f']);
    assert_eq!(
        stream,
        CharStream {
            chars: vec!['f', 'e', 'd', 'c', 'b', 'a']
        }
    );
    assert_eq!(stream.take(1)[0], 'a');
    assert_eq!(stream.len(), 5);
    assert_eq!(stream.test(|x| x.is_numeric()), Some(false));
    assert_eq!(stream.test_while(|x| x != 'd'), 3);
    assert_eq!(stream.collect(), vec!['b', 'c', 'd', 'e', 'f']);
}
