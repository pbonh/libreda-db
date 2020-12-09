
/// Signal type for pins.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    None,
    Input,
    Output,
    InOut,
    Clock,
    Supply,
    Ground,
}

impl Default for Direction {
    fn default() -> Self {
        Direction::None
    }
}

impl Direction {
    /// Check if this direction.rs is 'input'.
    pub fn is_input(&self) -> bool {
        self == &Direction::Input
    }
    /// Check if this direction.rs is 'output'.
    pub fn is_output(&self) -> bool {
        self == &Direction::Output
    }
}