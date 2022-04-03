use std::io::{self, Write};
use sozip::{SZEntry, Word, decode, encode, fill_dict, build_tree, inflate, deflate};

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

    let decoded: Vec<u8> = encoded.into_iter()
        .filter_map(|entry| decode(&tree, entry)).collect();
    assert_eq!(decoded.as_slice(), message);

    print!("Inflating... ");
    let buf = inflate(message, &tree);
    println!("{}b vs {}b", buf.len(), message.len());
    for entry in &buf {
        print!("{:#010b} ", entry);
    }
    println!();
    println!("Deflating...");
    let buf = deflate(buf.as_slice(), &tree);
    for entry in &buf {
        print!("{}", *entry as char);
    }
    println!();
    assert_eq!(buf.as_slice(), message);
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
