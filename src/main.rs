use std::fs::File;
use std::io::{BufReader, Read};

use crate::to_serde::serde_xml::parse_xml_file;
mod to_serde;

fn main() {
    let file = File::open("input.xml").expect("Failed to open file");
    let mut buf = String::new();
    BufReader::new(file)
        .read_to_string(&mut buf)
        .expect("to read from stdin");
    let x = parse_xml_file(&buf);

    println!("{x}");
}
