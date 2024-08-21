use std::fmt::Display;

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
    ClearEntry,
    Backspace,
}

impl From<&char> for Operator {
    fn from(value: &char) -> Self {
        match value {
            '+' => Self::Add,
            '-' => Self::Subtract,
            '*' => Self::Multiply,
            '÷' => Self::Divide,
            '%' => Self::Modulus,
            _ => panic!("Invalid operator"),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let symbol = match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "÷",
            Self::Modulus => "%",
            Self::Point => ".",
            Self::Equal => "=",
            Self::Clear => "C",
            Self::ClearEntry => "CE",
            Self::Backspace => "⌫",
        };

        write!(f, "{}", symbol)
    }
}
