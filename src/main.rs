// Entry point for WordlCrush

use std::fs;

fn load_allowed(fname: &str) -> Vec<String> {
    let raw_text_or_err = fs::read_to_string(fname);
    let mut o: Vec<String> = Vec::new();
    if let Ok(raw_text) = raw_text_or_err {
        let raw_list = raw_text.split("\n");
        for entry in raw_list {
            o.push(entry.trim().to_string());
        }
    } else {
        panic!("Cannot load {}", fname);
    }
    o
}

fn allowed_guesses_fname(args: Vec<String>) -> String {
    let mut arg_iter = args.iter();
    loop {
        let arg = arg_iter.next();
        if let Some(a) = arg {
            if a == "--file" {
                let file_arg = arg_iter.next();
                if let Some(f) = file_arg {
                    return f.to_string();
                } else {
                    break;
                }
            }
        } else {
            break;
        }
    }
    String::from("data/wordle-allowed-guesses.txt")
}

use std::env;

fn main() {
    // Rust does not apparently have a good standard way to do arg parsing other than blat them
    // into a Vec like it's 1999.
    let args: Vec<String> = env::args().collect();

    let wordl_allowed = load_allowed(&allowed_guesses_fname(args));
    println!("Loaded {} wordl allowed words", wordl_allowed.len());
}