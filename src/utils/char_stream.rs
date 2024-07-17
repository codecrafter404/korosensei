use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStream<T>
where
    T: Clone + PartialEq + Eq,
{
    available_items: Vec<T>,
    history: Vec<T>,
}
impl<T> ItemStream<T>
where
    T: Clone + PartialEq + Eq,
{
    pub fn prepend(&mut self, chars: Vec<T>) {
        let mut chars = chars;
        chars.reverse();
        self.available_items.extend_from_slice(&chars);
    }
    pub fn take(&mut self, n: usize) -> Vec<T> {
        let mut res = Vec::new();
        for _ in 0..n {
            if let Some(x) = self.available_items.pop() {
                self.history.push(x.clone());
                res.push(x);
            }
        }
        res
    }
    pub fn take_one(&mut self) -> Option<T> {
        self.take(1).into_iter().next()
    }
    pub fn get_history(&self) -> Vec<T> {
        self.history.clone()
    }
    pub fn take_while<F>(&mut self, test_function: F) -> Vec<T>
    where
        F: Fn(T) -> bool,
    {
        self.take(self.test_while(test_function))
    }
    pub fn test_window(&self, window: Vec<T>) -> Option<bool> {
        let mut dummy = self.clone();
        let test_window = dummy.take(3);
        if test_window.len() != window.len() {
            return None;
        }
        for (idx, w) in window.into_iter().enumerate() {
            if w != test_window[idx] {
                return Some(false);
            }
        }
        return Some(true);
    }
    pub fn preview(&self, n: usize) -> Vec<T> {
        let n = n.min(self.available_items.len());
        let mut res = Vec::new();
        for i in 0..n {
            res.push(self.available_items[self.available_items.len() - 1 - i].clone());
        }
        res
    }
    pub fn len(&self) -> usize {
        return self.available_items.len();
    }
    pub fn test_n<F>(&self, n: usize, test_function: F) -> Vec<bool>
    where
        F: Fn(T) -> bool,
    {
        let to_test = self.preview(n);
        to_test.into_iter().map(|x| test_function(x)).collect_vec()
    }
    pub fn test<F>(&self, test_function: F) -> Option<bool>
    where
        F: Fn(T) -> bool,
    {
        self.test_n(1, test_function).into_iter().next()
    }
    pub fn test_while<F>(&self, test_function: F) -> usize
    where
        F: Fn(T) -> bool,
    {
        let mut current: usize = 0;
        for x in self.available_items.iter().rev() {
            if !test_function(x.clone()) {
                break;
            }
            current += 1;
        }
        current
    }
    pub fn new(chars: &Vec<T>) -> Self {
        let mut chars = chars.clone();
        chars.reverse();
        ItemStream {
            available_items: chars,
            history: vec![],
        }
    }
    pub fn prev_collect(&self) -> Vec<T> {
        self.preview(self.available_items.len())
    }
    pub fn collect(&mut self) -> Vec<T> {
        self.take(self.available_items.len())
    }
    pub fn is_empty(&self) -> bool {
        self.available_items.is_empty()
    }
}

#[test]
fn test_char_stream() {
    let mut stream = ItemStream::new(&vec!['a', 'b', 'c', 'd', 'e', 'f']);
    assert_eq!(
        stream,
        ItemStream {
            available_items: vec!['f', 'e', 'd', 'c', 'b', 'a'],
            history: vec![]
        }
    );
    assert_eq!(stream.preview(1), vec!['a']);
    assert_eq!(stream.take(1)[0], 'a');
    assert_eq!(stream.len(), 5);
    assert_eq!(stream.test(|x| x.is_numeric()), Some(false));

    stream.prepend(vec!['a']);
    assert_eq!(stream.test_while(|x| x != 'd'), 3);
    assert_eq!(stream.prev_collect(), vec!['a', 'b', 'c', 'd', 'e', 'f']);
    assert_eq!(stream.test_window(vec!['a', 'b', 'c']), Some(true));
    assert_eq!(stream.test_window(vec!['a', 'b', 'x']), Some(false));
    assert_eq!(
        stream.test_window(vec!['a', 'b', 'x', 'x', 'x', 'x', 'x', 'x', 'x', 'x']),
        None
    );

    assert_eq!(stream.take_while(|x| x != 'd'), vec!['a', 'b', 'c']);
    assert_eq!(stream.test(|x| x == 'd'), Some(true));

    stream.prepend(vec!['h', 'i']);
    assert_eq!(stream.take(2), vec!['h', 'i']);

    assert_eq!(stream.collect(), vec!['d', 'e', 'f']);
    assert_eq!(
        stream.get_history(),
        vec!['a', 'a', 'b', 'c', 'h', 'i', 'd', 'e', 'f']
    );
    assert!(stream.is_empty())
}
