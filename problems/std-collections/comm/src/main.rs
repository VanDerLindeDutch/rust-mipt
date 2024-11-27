#![forbid(unsafe_code)]

// TODO: your code goes here.

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    // TODO: your code goes here.
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        panic!()
    }
    let file1 = File::open(args.get(1).unwrap()).unwrap();
    let file2 = File::open(args.get(2).unwrap()).unwrap();
    let mut set = HashSet::<String>::new();
    let reader = BufReader::new(file1);

    for line in reader.lines() {
        set.insert(line.unwrap());
    }
    let reader = BufReader::new(file2);
    let mut out = HashSet::<String>::new();
    for line in reader.lines() {
        let line = line.unwrap();
        if set.contains(&line) {
            out.insert(set.take(&line).unwrap());
        }
    }

    for line in out.iter() {
        println!("{}", line);
    }
}
