extern crate uuid;

pub mod commit;

pub fn inc_vec(vec: Vec<u8>) -> Vec<u8> {
    vec.iter().map(|x| x + 1).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
