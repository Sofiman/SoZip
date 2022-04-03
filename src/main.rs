use std::io::{self, Write};
use sozip::{SZEntry, Word, decode, encode, fill_dict, build_tree, inflate};

fn main() {
    print!("> ");
    io::stdout().flush().unwrap();

    let mut message = String::new();
    io::stdin().read_line(&mut message).unwrap();
    let message = message.trim().as_bytes();

    let mut dict = fill_dict(message);
    let tree = build_tree(&mut dict).unwrap();

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
    println!();
    let decoded: Vec<u8> = decoded.into_iter().flatten().collect();
    assert_eq!(decoded.as_slice(), message);
    let buf = inflate(message, &tree);
    for entry in buf {
        println!("{:#010b}", entry);
    }
}

#[allow(dead_code)]
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
