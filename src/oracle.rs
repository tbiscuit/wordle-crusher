// Oracle, holds the current secret. Returns the yellow / green / gray equivalent structure
// when asked to compare a guess to the secret.

#[derive(PartialEq, Eq, Clone)]
pub enum Square {
    GRAY,
    YELLOW,
    GREEN,
}

pub struct Reply {
    letters: Vec<Square>,
}

impl Reply {
    pub fn all_gray() -> Reply {
        let mut r: Vec<Square> = Vec::new();
        for i in 0..5 {
            r.push(Square::GRAY);
        }
        Reply {
            letters: r,
        }
    }

    pub fn with_set_light(&self, pos: usize, color: Square) -> Reply {
        let mut r: Vec<Square> = Vec::new();
        for i in 0..5 {
            if pos == i {
                r.push(color.clone());
            } else {
                r.push(self.letters[i].clone());
            }
        }
        Reply {
            letters: r
        }
    }

    pub fn light(&self, pos: usize) -> &Square {
        &self.letters[pos]
    }
}


pub struct Oracle {
    secret: String,
}

impl Oracle {
    pub fn create(s: &str) -> Oracle {
        Oracle {
            secret: String::from(s),
        }
    }

    pub fn compare(guess: &str, secret: &str) -> Reply {
        let mut r = Reply::all_gray();
        let mut used = Reply::all_gray();
        let mut g_chars = guess.chars();
        let mut s_chars = secret.chars();
        // first lock the green.
        for i in 0..5 {
            if let Some(g) = g_chars.next() {
                if let Some(s) = s_chars.next() {
                    if g == s {
                        r = r.with_set_light(i, Square::GREEN);
                        used = used.with_set_light(i, Square::GREEN);
                    }
                } else {
                    panic!("should not be able to happen!")
                }
            }else {
                panic!("also should not be able to happen!")
            }
        }
        // Now hunt for yellow.
        for i in 0..5 {
            if let Some(g) = g_chars.next() {
                if *r.light(i) == Square::GREEN {
                    continue;
                }
                for j in 0..5 {
                    if *used.light(j) != Square::GRAY {
                        continue;
                    }
                    if let Some(s) = s_chars.next() {
                        if s == g {
                            r = r.with_set_light(i, Square::YELLOW);
                            used = used.with_set_light(j, Square::YELLOW);
                            break;
                        }
                    }
                }
            } // we already panicked above for misaligned str
        }
        r
    }

    pub fn guess(&self, g: &str) -> Reply {
        Self::compare(g, &self.secret)
    }
}
