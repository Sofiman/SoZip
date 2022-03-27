use std::fmt;
use std::io::{self, Write};

type WordChild = Option<Box<Word>>;

#[derive(Debug)]
struct Word {
    value: u8,
    count: usize,
    is_empty: bool,
    left: WordChild,
    right: WordChild
}

impl Word {
    fn new(value: u8, count: usize) -> Self {
        Self {
            value, count,
            is_empty: false,
            left: None, right: None
        }
    }

    fn empty(count: usize) -> Self {
        Self {
            value: 0, count,
            is_empty: true,
            left: None, right: None
        }
    }

    fn tree(value: u8, count: usize, left: Word, right: Word) -> Self {
        Self {
            value, count,
            is_empty: true,
            left: Some(Box::new(left)), right: Some(Box::new(right))
        }
    }

    fn name(&self) -> String {
        if self.is_empty {
            format!("{}_{}", self.value, self.count)
        } else {
            format!("{}_{}", self.value, self.value as char)
        }
    }

    fn to_dot(&self) {
        let name = self.name();
        if let Some(left) = &self.left {
            println!("\"{}\" -- \"{}\";", name, left.name());
        }
        if let Some(right) = &self.right {
            println!("\"{}\" -- \"{}\";", name, right.name());
        }
        if let Some(left) = &self.left {
            left.to_dot();
        }
        if let Some(right) = &self.right {
            right.to_dot();
        }
    }

    fn value(&self) -> Option<u8> {
        if self.is_empty {
            None
        } else {
            Some(self.value)
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty {
            write!(f, "Ø({})", self.count)
        } else {
            write!(f, "'{}' -> {} ({})", self.value as char, self.value, self.count)
        }
    }
}

#[derive(Debug)]
struct SZEntry {
    value: usize,
    bits: usize
}

impl SZEntry {
    fn new(value: usize, bits: usize) -> Self {
        SZEntry { value, bits }
    }
}

impl fmt::Display for SZEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.bits {
            write!(f, "{}", (self.value >> i) & 0x1);
        }
        Ok(())
    }
}

fn main() {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut message = String::new();

    io::stdin().read_line(&mut message).unwrap();

    let message = message.trim().as_bytes();
    let mut dict = fill_dict(message);
    let tree = build_tree(&mut dict).unwrap();
    dump_dot(&tree);
    //print_tree(&tree, "", "");

    let encoded: Vec<SZEntry> = message.iter()
        .map(|c| encode(&tree, *c).unwrap()).collect();
    let mut total = 0;
    for entry in &encoded {
        total += entry.bits;
        print!("{}", entry);
    }
    println!();
    println!("in {}b vs {}b", 1 + ((total - 1) / 8), message.len());
    let decoded: Vec<Option<u8>> = encoded.into_iter()
        .map(|entry| decode(&tree, entry)).collect();
    for entry in &decoded {
        if let Some(c) = entry {
            print!("{}", *c as char);
        } else {
            print!("_");
        }
    }
    let decoded: Vec<u8> = decoded.into_iter().flatten().collect();
    assert_eq!(decoded.as_slice(), message);
}

fn dump_dot(tree: &Word) {
    println!("graph TREE {{");
    tree.to_dot();
    println!("}}");
}

fn insert_sorted(words: &mut Vec<Word>, value: Word){
    let len = words.len();
    let mut i = 0;
    while i < len && words[i].count > value.count {
        i += 1
    }
    words.insert(i, value);
}

fn fill_dict(data: &[u8]) -> Vec<Word> {
    let mut histogram: [usize; 256] = [0; 256];
    for b in data {
        histogram[*b as usize] += 1
    }
    let mut total = 0;
    let mut words = Vec::new();
    for (i, &count) in histogram.iter().enumerate() {
        if count > 0 {
            total += count;
            insert_sorted(&mut words, Word::new(i as u8, count));
        }
    }
    words.insert(0, Word::empty(total));

    let mut e = 0.0;
    let total = total as f64;
    for &count in histogram.iter().filter(|&count| *count > 0) {
        let p = 1.0 * (count as f64) / total;
        e -= p * p.log(256.0);
    }
    println!("entropy {}", e);


    words
}

fn build_tree(words: &mut Vec<Word>) -> Option<Word> {
    let mut count = words.len();
    while count > 3 {
       let right = words.pop()?;
       let left = words.pop()?;
       insert_sorted(words, 
           Word::tree(count as u8, left.count + right.count, left, right));
       count -= 1;
    }
    let first = words.remove(1);
    let second = words.remove(1);
    let root = &words[0];
    Some(Word::tree(0, root.count, first, second))
}

fn find_path(tree: &Word, search: u8, path: usize, bits: usize) 
    -> Option<SZEntry> {
    if tree.value == search {
        return Some(SZEntry::new(path, bits));
    }
    let mut result = None;
    if let Some(left) = &tree.left {
        result = find_path(left, search, path, bits + 1);
    }
    if result.is_none() {
        if let Some(right) = &tree.right {
            result = find_path(right, search, (1 << bits) + path, bits + 1);
        }
    }
    result
}

fn encode(tree: &Word, search: u8) -> Option<SZEntry> {
    find_path(tree, search, 0, 0)
}

fn follow_path(tree: &Word, path: usize, bits: usize) -> Option<u8> {
    if bits == 0 {
        return tree.value();
    }
    if (path & 1) == 0 {
        if let Some(left) = &tree.left {
            return follow_path(left, path >> 1, bits - 1);
        }
    } else if let Some(right) = &tree.right {
        return follow_path(right, path >> 1, bits - 1);
    }
    None
}

fn decode(tree: &Word, path: SZEntry) -> Option<u8> {
    follow_path(tree, path.value, path.bits)
}

fn print_tree(tree: &Word, branch: &str, path: &str) {
    print!("{} {}", branch, tree);
    if tree.left.is_none() && tree.right.is_none() {
        println!(" <{}>", path);
        return;
    }
    println!();
    let new_branch = branch.to_string() + "-";
    if let Some(left) = &tree.left {
        let new_path = path.to_string() + "0";
        print_tree(left, &new_branch, &new_path);
    }
    if let Some(right) = &tree.right {
        let new_path = path.to_string() + "1";
        print_tree(right, &new_branch, &new_path);
    }
}
