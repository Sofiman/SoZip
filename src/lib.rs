use std::fmt;

type WordChild = Option<Box<Word>>;

#[derive(Debug)]
pub struct Word {
    value: u8,
    count: usize,
    is_empty: bool,
    pub left: WordChild,
    pub right: WordChild
}

#[allow(dead_code)]
impl Word {
    pub fn new(value: u8, count: usize) -> Self {
        Self {
            value, count,
            is_empty: false,
            left: None, right: None
        }
    }

    pub fn empty(count: usize) -> Self {
        Self {
            value: 0, count,
            is_empty: true,
            left: None, right: None
        }
    }

    pub fn tree(value: u8, count: usize, left: Option<Word>, right: Option<Word>) -> Self {
        Self {
            value, count,
            is_empty: true,
            left:   left.map(Box::new),
            right: right.map(Box::new)
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

    pub fn value(&self) -> Option<u8> {
        if self.is_empty {
            None
        } else {
            Some(self.value)
        }
    }

    pub fn dump_dot(&self) {
        println!("graph TREE {{");
        self.to_dot();
        println!("}}");
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
pub struct SZEntry {
    pub value: usize,
    pub bits: usize
}

impl SZEntry {
    pub fn new(value: usize, bits: usize) -> Self {
        SZEntry { value, bits }
    }
}

impl fmt::Display for SZEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.bits {
            write!(f, "{}", (self.value >> i) & 0x1)?;
        }
        Ok(())
    }
}

fn insert_sorted(words: &mut Vec<Word>, value: Word){
    let len = words.len();
    let mut i = 0;
    while i < len && words[i].count > value.count {
        i += 1
    }
    words.insert(i, value);
}

pub fn fill_dict(data: &[u8]) -> Vec<Word> {
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
    /*
    let mut e = 0.0;
    let total = total as f64;
    for &count in histogram.iter().filter(|&count| *count > 0) {
        let p = 1.0 * (count as f64) / total;
        e -= p * p.log(256.0);
    }
    println!("entropy {}", e);
    */

    words
}

pub fn build_tree(words: &mut Vec<Word>) -> Option<Word> {
    let mut count = words.len();

    if count == 0 {
        return None;
    }

    while count > 3 {
       let right = words.pop()?;
       let left = words.pop()?;
       insert_sorted(words, Word::tree(count as u8, left.count + right.count, 
           Some(left), Some(right)));
       count -= 1;
    }

    let second = words.pop();
    let first = words.pop();
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

pub fn encode(tree: &Word, search: u8) -> Option<SZEntry> {
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

pub fn decode(tree: &Word, path: SZEntry) -> Option<u8> {
    follow_path(tree, path.value, path.bits)
}

pub fn inflate(in_buffer: &[u8], tree: &Word) -> Vec<u8> {
    let mut out_buf = Vec::new();
    let mut acc = 0;
    let mut filled_bits: usize = 3; // 3 bits for total_bits mod 8

    // FIXME: Panic when encoded.value exeeds 8 bits (subtract with overflow
    // when getting the remaining number of bits)
    for b in in_buffer {
        let encoded = encode(tree, *b).unwrap();

        let to_fill = 8 - filled_bits;
        if to_fill < encoded.bits {
            // select the last n bits to fill the current byte
            let mask = 1 << to_fill;
            acc |= (encoded.value & (mask - 1)) << filled_bits;
            out_buf.push(acc as u8);

            // keep the last (8-n) bytes and put it into acc
            acc = encoded.value >> to_fill;
            filled_bits = encoded.bits - to_fill;
        } else {
            acc += encoded.value << filled_bits;
            filled_bits += encoded.bits;
        }
    }

    if filled_bits != 0 {
        out_buf.push(acc as u8);
    }

    // put the number of remaining bits at the last byte in the 3 first 
    // bits of the first byte
    out_buf[0] = ((out_buf[0] as usize) | (filled_bits % 8)) as u8;
    out_buf
}

pub fn deflate(in_buffer: &[u8], tree: &Word) -> Vec<u8> {
    let mut out_buf = Vec::new();
    let len = in_buffer.len();
    if len == 0 {
        return out_buf
    }

    let mut i: usize = 3; // skip first 3 bits

    let last_bits = in_buffer[0] & 0b111;
    let len = (len-1)*8 + last_bits as usize; // exact number of bits in message
    let mut current_node = tree;

    while i < len {
        let idx = i / 8;
        let cursor = i % 8;

        let b = in_buffer[idx];
        let dir = (b >> cursor) & 1;

        if dir == 0 { // follow path
            current_node = current_node.left.as_ref().unwrap();
        } else {
            current_node = current_node.right.as_ref().unwrap();
        }

        if !current_node.is_empty { // found a leaf
            out_buf.push(current_node.value);
            current_node = tree;
        }

        i += 1;
    }

    out_buf
}
