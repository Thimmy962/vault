#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_mut)]
mod bytes;
mod debug;
mod scanner;
mod token;
mod compiler;
mod vm;
mod error;
mod string;

// static config: Config;

use std::{env, fs, io::{self, Write}, process};
use scanner::Scanner;
use bytes::Bytes;

use crate::{string::Obj, vm::VM};

fn main() {
    let config = Config::new(); // read command line arguments
    
    if config.file_given {
        non_shell(&config);
    } else {
        shell(&config);
    }
}


fn non_shell(config: &Config) {
    let mut file = fs::read_to_string(&config.filename).unwrap_or_else(|err| {
        match err.kind() {
            io::ErrorKind::NotFound => {
                println!("File with name '{}' not found", {&config.filename});
                process::exit(1);
            }
            _ => {
                println!("Error: {err}");
                process::exit(1);
            }
        }
    });
    file += "\0";
    let mut scanner = Scanner::build(file);
    let mut chunk = bytes::Bytes::new();
    let mut objects: Option<*mut Obj> = None;

    let cmpiler = compiler::compile(&mut scanner, &mut chunk, &mut objects);
    let mut vm = VM::build(objects);
    if config.debug {
        let debugger = debug::Debug::new(&chunk);
        if !cmpiler {
            debugger.run();
        }
    }
    else if !cmpiler { // if there is no error with the compiler run vm
        vm.run(&chunk);
        chunk.clear();
    }
}

fn shell(_config: &Config) {
    let mut _chunk = Bytes::new();
    let mut input = String::new();
    let mut objects: Option<*mut Obj> = None;
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
        input+="\0";

        // pass a reference of the input to the scanner
        let mut scanner = Scanner::build((&input).to_string());

        let cmpiler = compiler::compile(&mut scanner, &mut _chunk, &mut objects);
    
        let mut vm = VM::build(objects);

        if _config.debug {
            let debugger = debug::Debug::new(&_chunk);
            if !cmpiler {
                debugger.run();
                }
            return;
        }
        //retru
        if !cmpiler {
            vm.run(&_chunk);
        }
        input.clear();
        _chunk.clear();

    }

}



struct Config {
    debug: bool,
    filename: String, // the name of the file containing the main program
    file_given: bool // if a file was given
}

impl Config {
    fn new() -> Config {
        let mut filename = "".to_string();
        let mut debug = false;
        let mut file_g = false;
            for arg in env::args() {
                if "--debug" == arg || "-d" == arg {
                    debug = true;
                    continue;
                }

            // .vlt is the name of the file that contains the main function that is the only file that
            // should be passed to the compiler, every other file(".vt") should be called directly
            // or indirectly by main function
            else if arg.contains(".vlt")  {
                filename = arg;
                file_g = true;
                continue;
            }

            if env::args().len() > 1 {
                filename = arg;
                file_g = true;
            }
        }
        Config { 
            debug, 
            file_given: file_g,
            filename: filename
        }
    }
}

