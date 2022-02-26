// Solver, uses the allowed words and a secret word to try to guess.

pub struct Solver {
    allowed: Vec<String>,
    possible: Vec<String>,
    loud_mode: bool,
}

use crate::Oracle;
use crate::Reply;
use std::collections::HashMap;

impl Solver {
    pub fn create(a: Vec<String>, p: Vec<String>) -> Solver{
        Solver {
            allowed: a,
            possible: p,
            loud_mode: true,
        }
    }

    pub fn solve(&self, oracle: Oracle) -> Vec<String> {
        let mut known_possible = self.possible.clone();
        let mut o: Vec<String> = Vec::new();
        let mut death_clock = 0;
        loop {
            let good_guess = Self::calculate_guess(&self.allowed, &known_possible);
            o.push(good_guess.clone());
            let reply = oracle.guess(&good_guess);
            if reply.all_green() {
                break;
            }
            known_possible = Self::reduce_set(&good_guess, reply, &known_possible, self.loud_mode);
            death_clock += 1;
            if death_clock > 10 {
                panic!("Solver is terrible and has been sacked!");
            }
        }
        o
    }

    fn calculate_guess(_allowed: &Vec<String>, possible: &Vec<String>) -> String {
        if let Some(w) = possible.iter().next() {
            return String::from(w);
        } else {
            panic!("should not be out of words to guess!");
        }
    }

    fn reduce_set(guess: &str, reply: Reply, possible: &Vec<String>, loud: bool) -> Vec<String> {
        let mut known_possible = Vec::new();
        for word in possible {
            if Self::is_feasible(guess, &reply, &word) {
                known_possible.push(word.clone());
            }
        }
        if loud {
            print!("Guessed {}, Got {}, Reduced set from {} entries to {{", guess, reply.to_string(), possible.len());
            for kp in known_possible.iter() {
                print!("{}, ", kp);
            }
            println!("}}");
        }
        known_possible
    }

    fn is_feasible(guess: &str, reply: &Reply, word: &str) -> bool {
        if guess == word {
            return false; // we know the current guess is not a good next guess
        }
        // are the green ones right?
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
            }
        }
        // Yellow is tricky because dups are allowed.
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
            let w_get = guess_map.get(&g);
            if let Some(n) = g_get {
                match w_get {
                    None => {
                        // Word clearly does not match constraint.
                        return false;
                    },
                    Some(m) => {
                        if m != n {
                            // Word contains the letter but in an incorrect quantity.
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