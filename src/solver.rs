// Solver, uses the allowed words and a secret word to try to guess.

pub struct Solver {
    allowed: Vec<String>,
    possible: Vec<String>,
    loud_mode: bool,
    thread_count: u32,
    max_search: usize,
}

use crate::Oracle;
use crate::Reply;
use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, Mutex};

impl Solver {
    pub fn create(a: Vec<String>, p: Vec<String>, tc: u32, ms: u32, l: bool) -> Solver{
        println!("Solver created with {} threads, {} length to begin exhaustive search, verbose = {}", tc, ms, l);
        Solver {
            allowed: a,
            possible: p,
            loud_mode: l,
            thread_count: tc,
            max_search: ms as usize,
        }
    }

    pub fn solve(&self, oracle: Oracle) -> Vec<String> {
        let mut known_possible = self.possible.clone();
        let mut o: Vec<String> = Vec::new();
        let mut death_clock = 0;
        loop {
            let good_guess = Self::calculate_guess(&self.allowed, &known_possible, self.thread_count, self.max_search);
            o.push(good_guess.clone());
            let reply = oracle.guess(&good_guess);
            if reply.all_green() {
                break;
            }
            known_possible = Self::reduce_set(&good_guess, reply, &known_possible, self.loud_mode);
            death_clock += 1;
            if death_clock > 12 {
                panic!("Solver is terrible and has been sacked!");
            }
        }
        o
    }

    fn calculate_guess(allowed: &Vec<String>, possible: &Vec<String>, tc: u32, ms: usize) -> String {
        let mut best_word = String::new();
        let mut golf_score = possible.len();
        // If there is a better chance by guessing one of the few remaining words or
        // the number possible is too large for exhaustive search to be worth the time.
        if possible.len() < 3 || possible.len() > ms {
            // Then just guess the first from possibles.
            let o = possible.iter().next();
            if let Some(w) = o {
                return String::from(w);
            } else {
                panic!("Failed to guess!");
            }
        }
        for word in allowed {
            // Unfortunately, thread creation is rather expensive in Rust.
            // Limit the use of the multithreaded version to cases where it will likely help.
            if possible.len() > tc as usize * 2 {
                let hi_score_mut = Arc::new(Mutex::new(0 as usize));
                let mut join_handles = Vec::new();
                let lifetime_possibles: Vec<String> = possible.clone();
                let shared_possibles = Arc::new(lifetime_possibles);
                let tcu = tc as usize;
                for offset in 0..tc {
                    let child_possibles = Arc::clone(&shared_possibles);
                    let movable_word = word.clone();
                    let hi_score_shared_mut = hi_score_mut.clone();
                    join_handles.push(thread::spawn(move || {
                        let mut i = offset as usize;
                        while i < child_possibles.len() {
                            let p = &child_possibles[i];
                            let r = Oracle::compare(&movable_word, p);
                            let reduced = Self::reduce_set(&movable_word, r, &child_possibles,
                                                           false);
                            let mut hi_score = hi_score_shared_mut.lock().unwrap();
                            if reduced.len() > *hi_score {
                                *hi_score = reduced.len();
                            }
                            std::mem::drop(hi_score); // manual unlock is needed in loops like this
                            i += tcu;
                        }
                    }));
                }
                // Hold here for a sec...
                for handle in join_handles.into_iter() {
                    handle.join().unwrap();
                }
                // mutex will unlock itself by going out of scope
                let hi_score = hi_score_mut.lock().unwrap();
                if *hi_score <= golf_score {
                    golf_score = *hi_score;
                    best_word = word.clone();
                }
            } else {
                let mut hi_score = 0 as usize;
                for p in possible {
                    let r = Oracle::compare(&word, &p);
                    let reduced = Self::reduce_set(&word, r, &possible, false);
                    if reduced.len() > hi_score {
                        hi_score = reduced.len();
                    } 
                }
                if hi_score <= golf_score {
                    golf_score = hi_score;
                    best_word = word.clone();
                }
            }
        }
        if best_word.is_empty() {
            panic!("This algorithm doesn't pick words!");
        }
        best_word
    }

    fn reduce_set(guess: &str, reply: Reply, possible: &Vec<String>, loud: bool) -> Vec<String> {
        let mut known_possible = Vec::new();
        for word in possible {
            if Self::is_feasible(guess, &reply, &word) {
                known_possible.push(word.clone());
            }
        }
        if loud {
            print!("Guessed {}, Got {}, Reduced set from {} entries to ", guess, reply.to_string(), possible.len());
            if known_possible.len() < 10 {
                print!("{{");
                for kp in known_possible.iter() {
                    print!("{}, ", kp);
                }
                println!("}}");
            } else {
                println!("{} entries", known_possible.len());
            }
        }
        known_possible
    }

    fn is_feasible(guess: &str, reply: &Reply, word: &str) -> bool {
        if guess == word {
            return false; // we know the current guess is not a good next guess
        }
        // Green and basic yellow elimination.
        let mut g_iter = guess.chars();
        let mut w_iter = word.chars();
        for i in 0..5 {
            let cur_g = g_iter.next();
            let cur_w = w_iter.next();
            if reply.is_green(i) {
                if let Some(g) = cur_g {
                    if let Some(w) = cur_w {
                        if g != w {
                            return false
                        }
                    }
                }
            } else if reply.is_yellow(i) {
                // Basic yellow elimination, e.g., a yellow r in position 1 means an r cannot be there.
                if let Some(g) = cur_g {
                    if let Some(w) = cur_w {
                        if g == w {
                            return false
                        }
                    }
                }
            }        
        }
            }
        }
        // Yellow also gets us letter counts to help eliminate some words with duplicate letters,
        // or to eliminate words that do NOT have duplicate letters that we know to exist.
        g_iter = guess.chars();
        let mut guess_map: HashMap<char, u8> = HashMap::new();
        for i in 0..5 {
            let cur_g = g_iter.next();
            if !reply.is_gray(i) { // count green and yellow so that we have a true count of how
                                   // many copies of the letter there are.
                if let Some(g) = cur_g {
                    let e = guess_map.entry(g).or_insert(0);
                    *e += 1;
                }
            }
        }
        w_iter = word.chars();
        let mut word_map: HashMap<char, u8> = HashMap::new();
        for w in w_iter {
            let e = word_map.entry(w).or_insert(0);
            *e += 1;
        }
        g_iter = guess.chars();
        for g in g_iter { // only iterate over constraints provided by the reply to the guess.
            let g_get = guess_map.get(&g);
            let w_get = word_map.get(&g);
            if let Some(n) = g_get {
                match w_get {
                    None => {
                        // Word clearly does not match constraint.
                        return false;
                    },
                    Some(m) => {
                        if m < n {
                            // Word should contain at least n copies of this letter.
                            return false;
                        }
                    }
                }
            }
        }
        // Finally eliminate based on gray
        g_iter = guess.chars();
        for i in 0..5 {
            let cur_g = g_iter.next();
            if reply.is_gray(i) {
                if let Some(g) = cur_g {
                    if word_map.contains_key(&g) && !guess_map.contains_key(&g) {
                        // We have a gray letter and no yellow or green squares with that letter,
                        // since it appears in the candidate word, that word cannot be the
                        // solution.
                        return false;
                    }
                }
            }
        }
        true
    }
}