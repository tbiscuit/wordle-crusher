// Solver, uses the allowed words and a secret word to try to guess.

pub struct Solver {
    allowed: Vec<String>,
    possible: Vec<String>,
    loud_mode: bool,
    max_search: usize,
}

struct MapOut {
    hi_score: usize,
    word: String,
}

use crate::Oracle;
use crate::Reply;
use std::collections::HashMap;
use rayon::prelude::*;

impl Solver {
    pub fn create(a: Vec<String>, p: Vec<String>, ms: u32, l: bool) -> Solver{
        println!("Solver created with {} length to begin exhaustive search, verbose = {}", ms, l);
        Solver {
            allowed: a,
            possible: p,
            loud_mode: l,
            max_search: ms as usize,
        }
    }

    pub fn solve(&self, oracle: Oracle) -> Vec<String> {
        let mut known_possible = self.possible.clone();
        let mut o: Vec<String> = Vec::new();
        let mut death_clock = 0;
        loop {
            let good_guess = Self::calculate_guess(&self.allowed, &known_possible, self.max_search);
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

    fn calculate_guess(allowed: &Vec<String>, possible: &Vec<String>, ms: usize) -> String {
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
        let best = allowed.par_iter().map(|word| {
            let mut out = MapOut {
                hi_score: 0,
                word: String::from(word)
            };
            for p in possible {
                let r = Oracle::compare(&word, &p);
                let reduced = Self::reduce_set(&word, r, &possible, false);
                if reduced.len() > out.hi_score {
                    out.hi_score = reduced.len();
                } 
            }
            out
        }).reduce(|| MapOut { hi_score: possible.len() + 1, word: String::from("")}, |a, b| {
            if a.hi_score < b.hi_score {
                a
            } else {
                b
            }
        });
        if best.word.is_empty() {
            panic!("This algorithm doesn't pick words!");
        }
        best.word
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
        if reply.all_green() {
            // can happen during speculative search.
            return true;
        }
        // Basic elimination, all colors have a basic fact they reveal.
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
            } else {
                // Basic gray elimination ask about a letter in exactly this place ange got gray?
                if let Some(g) = cur_g {
                   if let Some(w) = cur_w {
                     if g == w {
                            return false;
                        }
                    }
                }
            }
        }
        // Create hashmaps, used for gray and dup letter based elimination.
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
        // Eliminate based on gray
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
        // Finally, try come more complex elimination based on using Yellow + Green together
        // to eliminate words if we can determine that the word has the wrong number of
        // duplicates of a duplicated letter.
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
        true
    }
}
