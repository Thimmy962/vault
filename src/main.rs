mod bytes;
mod debug;
mod scanner;
mod token;

use std::{env, io::{self, Write}, process};
use debug::Debug;
use scanner::Scanner;

use crate::bytes::Bytes;

fn main() {
    let config = Config::new(); // read command line arguments
    let mut chunk = Bytes::new();
    
    let mut input = String::new();

    loop {
        print!(":> ");
        let _ = io::stdout().flush();

        // read input. If there is an error in read, stop the entire program
        io::stdin().read_line(&mut input).unwrap_or_else(|err| {
            println!("{err}");
            process::exit(1);

        });
        
        if "exit" == input.trim() {
            process::exit(0);
        }

        // pass a reference of the input to the scanner
        let mut scanner = Scanner::build(&input);
        scanner.scan_token().print_token();
        input.clear();
    }

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
