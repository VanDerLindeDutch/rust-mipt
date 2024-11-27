#![forbid(unsafe_code)]

use std::collections::VecDeque;
use itertools::Itertools;
pub fn combinations(arr: &[i32], k: usize) -> Vec<Vec<i32>> {
    if arr.is_empty() || k == 0 {
        return vec![vec![]];
    }
    let mut out = Vec::<Vec<i32>>::new();
    cmb(&arr, k, 1, Vec::new(), &mut out);

    return out;
}

fn cmb(arr: &[i32], k:usize, cur: usize,mut buf: Vec<i32>, mut out: &mut Vec<Vec<i32>>) {
    if buf.len() == k {
        out.push(buf);
        return;
    }

    for x in arr.iter().enumerate() {
        let mut to_send = buf.clone();
        to_send.push(*x.1);
        cmb(&arr[x.0+1..], k,1, to_send.clone(), &mut out);

    }

}