// Entry point for WordlCrush

use wordl_crush::{Solver};

use std::fs;

#[macro_export]
macro_rules! path_up {
    ($deflt: expr) => {
        if true {
            let def_root = std::env::current_dir();
            if let Ok(base_path) = def_root {
                // Note: This could fail for a sufficiently annoying OS.
                let maybe_path = base_path.to_str();
                if let Some(def_path) = maybe_path {
                    let mut out_path: String = String::from(def_path);
                    out_path.push('/');
                    out_path.push_str($deflt);
                    out_path
                } else {
                    String::from($deflt)
                }
            } else {
                String::from($deflt)
            }
        } else {
            String::from($deflt)
        }
    }
}

fn load_list_from_file(fname: &str) -> Vec<String> {
    let raw_text_or_err = fs::read_to_string(fname);
    let mut o: Vec<String> = Vec::new();
    if let Ok(raw_text) = raw_text_or_err {
        let raw_list = raw_text.split("\n");
        for entry in raw_list {
            o.push(entry.trim().to_string());
        }
    } else {
        let open_char = fname.chars().next().unwrap();
        if open_char == '/' || open_char == '\\' {
            panic!("Cannot load {}", fname);
        } else {
            panic!("Cannot load (implied path) {}", path_up!(fname));
        }
    }
    o
}

#[macro_export]
macro_rules! load_wlist {
    ($arg_iter: expr, $flag: expr, $deflt: expr) => {
        loop {
            let arg = $arg_iter.next();
            if let Some(a) = arg {
                if a == $flag {
                    let file_arg = $arg_iter.next();
                    if let Some(f) = file_arg {
                        return f.to_string();
                    } else {
                        return String::from($deflt);
                    }
                }
            } else {
                return String::from($deflt);
            }
        }
    }
}

fn allowed_guesses_fname(args: Vec<String>) -> String {
    let mut arg_iter = args.iter();
    load_wlist!(arg_iter, "--allowed", "data/wordle-allowed-guesses.txt");
}
fn possible_guesses_fname(args: Vec<String>) -> String {
    let mut arg_iter = args.iter();
    load_wlist!(arg_iter, "--possible", "data/wordle-possible-solutions.txt");
}

use std::env;

fn main() {
    // Rust does not apparently have a good standard way to do arg parsing other than blat them
    // into a Vec like it's 1999.
    let args: Vec<String> = env::args().collect();

    let wordl_allowed = load_list_from_file(&allowed_guesses_fname(args.clone()));
    let wordl_possible = load_list_from_file(&possible_guesses_fname(args));
    println!("Loaded {} wordl allowed words, with {} possible solutions",
             wordl_allowed.len(), wordl_possible.len());
    let try_words = wordl_possible.clone();
    for word in try_words {
        let solver = Solver::create(wordl_allowed.clone(), wordl_possible.clone());
        let guesses = solver.solve(word.clone());
        println!("used {} guesses to solve {}", guesses.len(), word);
    }
}