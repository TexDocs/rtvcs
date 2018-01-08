#![feature(try_from)]

#[cfg(test)]
extern crate rand;
#[macro_use]
extern crate stdweb;
extern crate uuid;

use stdweb::unstable::TryInto;
use stdweb::Value;

mod commit;

fn main() {
    stdweb::initialize();

    // js! {
    //     Module.exports.incrementArray = @{inc_vec}
    // }
}
