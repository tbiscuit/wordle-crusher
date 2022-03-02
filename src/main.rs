// Entry point for WordlCrush

use wordle_crush::{Solver, Oracle};

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
            let word = entry.trim().to_string();
            if word.len() == 5 {
                o.push(word);
            } else {
                println!("{} found in file and ignored!", word);
            }
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
macro_rules! extract_arg {
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

fn allowed_guesses_fname(args: &Vec<String>) -> String {
    let mut arg_iter = args.iter();
    extract_arg!(arg_iter, "--allowed", "data/wordle-allowed-guesses.txt");
}
fn possible_guesses_fname(args: &Vec<String>) -> String {
    let mut arg_iter = args.iter();
    extract_arg!(arg_iter, "--possible", "data/wordle-possible-solutions.txt");
}

fn max_search_arg(args: &Vec<String>) -> String {
    let mut arg_iter = args.iter();
    extract_arg!(arg_iter, "--max_search", "30");
}

fn loud_mode(args: &Vec<String>) -> bool {
    for arg in args.iter() {
        if arg == "--verbose" {
            return true;
        }
    }
    false
}

fn hard_mode(args: &Vec<String>) -> bool {
    for arg in args.iter() {
        if arg == "--hard" {
            return true;
        }
    }
    false
}

use std::env;
use std::collections::HashMap;

fn main() {
    // Rust does not apparently have a good standard way to do arg parsing other than blat them
    // into a Vec like it's 1999.
    let args: Vec<String> = env::args().collect();

    let wordl_allowed = load_list_from_file(&allowed_guesses_fname(&args));
    let wordl_possible = load_list_from_file(&possible_guesses_fname(&args));
    let verbose = loud_mode(&args);
    let hard_mode = hard_mode(&args);
    println!("Loaded {} wordl allowed words, with {} possible solutions",
             wordl_allowed.len(), wordl_possible.len());
    let try_words = wordl_possible.clone();
    let msa = max_search_arg(&args).parse::<u32>();
    let mut ms: u32 = 30;
    if let Ok(n) = msa {
        ms = n;
    }
    let solver = Solver::create(wordl_allowed.clone(), wordl_possible.clone(), ms, hard_mode, verbose);
    let mut histogram: HashMap<usize, u32> = HashMap::new();
    let mut total_guesses: u32 = 0;
    let mut max_for_one_word: u32 = 0;
    for word in try_words {
        let oracle = Oracle::create(&word);
        let guesses = solver.solve(oracle);
        println!("used {} guesses to solve {}", guesses.len(), word);
        total_guesses += guesses.len() as u32;
        if max_for_one_word < guesses.len() as u32 {
            max_for_one_word = guesses.len() as u32;
        }
        let e = histogram.entry(guesses.len()).or_insert(0);
        *e += 1;
    }
    let mut counted: u32 = 0;
    let mut median: u32 = 0;
    let average = total_guesses as f32 / wordl_possible.len() as f32;
    for i in 0..max_for_one_word {
        let entry = histogram.get(&((i+1) as usize));
        if let Some(e) = entry {
            println!("Got {} words in {} guesses", e, i+1);
            counted += e * (i+1);
            if median == 0 && counted > total_guesses / 2 {
                median = i+1;
            }
        }
    }
    println!("Total Guesses {}", total_guesses);
    println!("Total Words {}", wordl_possible.len());
    println!("Average guesses: {}", average);
    println!("Median guesses: {}", median);
}