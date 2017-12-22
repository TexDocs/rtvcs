#![feature(try_from)]

#[macro_use] extern crate stdweb;

use stdweb::unstable::TryInto;
use stdweb::Value;

mod lib;

fn inc_vec(vec: Value) -> Value {
    lib::inc_vec(vec.try_into().unwrap()).try_into().unwrap()
}

fn main() {
    stdweb::initialize();

    js! {
        Module.exports.incrementArray = @{inc_vec}
    }
}
