mod bytes;
mod debug;

use std::env;
use debug::Debug;

use crate::bytes::Bytes;

fn main() {
    let config = Config::new(); // read command line arguments
    let mut chunk = Bytes::new();
    chunk.write_constant(bytes::Value::DOUBLE(21.0));
    chunk.write_constant(bytes::Value::INT(21));
    chunk.write_constant(bytes::Value::BOOL(false));
    chunk.write_constant(bytes::Value::NIL);
    if config.debug {
        let debug = Debug::new(&chunk);
        debug.run();
    }

}

struct Config {
    debug: bool,
}

impl Config {
    fn new() -> Config {
        for arg in env::args() {
            if "--debug" == arg || "-d" == arg {
                return Config {debug: true}
            }
        }
        Config { debug: false }
    }
}
