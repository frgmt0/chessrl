pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

impl Move {
    pub fn new(from: (usize, usize), to: (usize, usize)) -> Self {
        Move { from, to }
    }
}
