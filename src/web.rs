#![feature(try_from)]

#[macro_use]
extern crate stdweb;
extern crate uuid;
#[cfg(test)]
extern crate rand;

use stdweb::unstable::TryInto;
use stdweb::Value;

mod commit;

fn main() {
    stdweb::initialize();

    // js! {
    //     Module.exports.incrementArray = @{inc_vec}
    // }
}
