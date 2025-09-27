#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Point,
    Equal,
    Clear,
    Backspace,
    ParenthesesOpen,
    ParenthesesClose,
    Power,
    SquareRoot,
}

impl Operator {
    pub fn display(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "x",
            Self::Divide => "÷",
            Self::Modulus => "%",
            Self::Point => ".",
            Self::Equal => "=",
            Self::ParenthesesOpen => "(",
            Self::ParenthesesClose => ")",
            Self::Power => "^",
            Self::SquareRoot => "√",
            Self::Clear => "C",
            Self::Backspace => "⌫",
        }
    }

    pub fn expression(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulus => "%",
            Self::Point => ".",
            Self::Equal => "=",
            Self::ParenthesesOpen => "(",
            Self::ParenthesesClose => ")",
            Self::Power => "^",
            Self::SquareRoot => "√",
            Self::Clear => "C",
            Self::Backspace => "⌫",
        }
    }
}
