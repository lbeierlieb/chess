#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    fn to_x_y(&self) -> (i8, i8) {
        match self {
            Direction::North => (0, 1),
            Direction::NorthEast => (1, 1),
            Direction::East => (1, 0),
            Direction::SouthEast => (1, -1),
            Direction::South => (0, -1),
            Direction::SouthWest => (-1, -1),
            Direction::West => (-1, 0),
            Direction::NorthWest => (-1, 1),
        }
    }

    pub fn all_non_diagonal() -> Vec<Self> {
        vec![
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }

    pub fn all_diagonal() -> Vec<Self> {
        vec![
            Direction::NorthEast,
            Direction::SouthEast,
            Direction::SouthWest,
            Direction::NorthWest,
        ]
    }

    pub fn all() -> Vec<Self> {
        let mut dirs = Self::all_non_diagonal();
        dirs.append(&mut Self::all_diagonal());
        dirs
    }

    pub fn is_same_axis(&self, other: &Direction) -> bool {
        let (x, y) = self.to_x_y();
        let (xo, yo) = other.to_x_y();
        x == xo && y == yo || x == -xo && y == -yo
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self::new_checked(x, y).expect(&format!("({}, {}) is not a valid coordinate", x, y))
    }

    pub fn new_checked(x: u8, y: u8) -> Option<Self> {
        if x <= 7 && y <= 7 {
            Some(Self { x, y })
        } else {
            None
        }
    }

    pub fn from_str(text: &str) -> Self {
        if text.len() != 2 {
            panic!();
        }
        let chars = text.chars().collect::<Vec<_>>();
        if chars.len() != 2 {
            panic!();
        }
        let x = (chars[0] as u8).wrapping_sub('A' as u8);
        let y = (chars[1] as u8).wrapping_sub('1' as u8);
        Self::new_checked(x, y).expect(&format!("'{}' is not a valid coordinate", text))
    }

    pub fn moved(&self, dir: Direction, amount: i8) -> Option<Self> {
        let (xdir, ydir) = dir.to_x_y();
        let x = self.x.checked_add_signed(xdir.checked_mul(amount)?)?;
        let y = self.y.checked_add_signed(ydir.checked_mul(amount)?)?;
        Self::new_checked(x, y)
    }
}
