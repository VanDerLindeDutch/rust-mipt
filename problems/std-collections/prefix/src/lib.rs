#![forbid(unsafe_code)]

use std::cmp::min;

pub fn longest_common_prefix(strs: Vec<&str>) -> String {
    if strs.len() == 0 {
        return String::new();
    }
    let mut out = strs.get(0).unwrap().clone().as_bytes();
    let mut skip = true;
    for s in strs{
        if skip {
            skip = false;
            continue
        }
        let l = min(out.len(), s.len());
        let mut i = 0;
        let mut sIter = s.as_bytes();
        let mut outChars = out;
        while i <l {
            if !sIter[i].eq(&outChars[i]){
                out = &out[..i];
                break;
            }
            i+=1;
        }
    }
    // TODO: your code goes here.
    String::from_utf8(out.to_vec()).unwrap()
}
