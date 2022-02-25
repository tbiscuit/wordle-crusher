// Solver, uses the allowed words and a secret word to try to guess.

pub struct Solver {
    allowed: Vec<String>,
    possible: Vec<String>,
}

impl Solver {
    pub fn create(a: Vec<String>, p: Vec<String>) -> Solver{
        Solver {
            allowed: a,
            possible: p,
        }
    }

    pub fn solve(&self, _secret: String) -> Vec<String> {
        let o: Vec<String> = Vec::new();
        o
    }
}