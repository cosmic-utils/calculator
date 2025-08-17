#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    StartGroup,
    EndGroup,
    Pi,
    Root,
    Square,
    Percentage,
    Point,
    Equal,
    Clear,
    Backspace,
}

impl Operator {
    pub fn display(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "x",
            Self::Divide => "÷",
            Self::Modulus => "mod",
            Self::Point => ".",
            Self::Equal => "=",
            Self::Clear => "C",
            Self::Backspace => "⌫",
            Self::StartGroup => "(",
            Self::EndGroup => ")",
            Self::Pi => "π",
            Self::Root => "√",
            Self::Square => "²",
            Self::Percentage => "%",
        }
    }

    pub fn button_display(&self) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "x",
            Self::Divide => "÷",
            Self::Modulus => "mod",
            Self::Point => ".",
            Self::Equal => "=",
            Self::Clear => "C",
            Self::Backspace => "⌫",
            Self::StartGroup => "(",
            Self::EndGroup => ")",
            Self::Pi => "π",
            Self::Root => "√",
            Self::Square => "x²",
            Self::Percentage => "%",
        }
    }

    pub fn expression(&self) -> String {
        match self {
            Self::Add => "+".to_string(),
            Self::Subtract => "-".to_string(),
            Self::Multiply => "*".to_string(),
            Self::Divide => "/".to_string(),
            Self::Point => ".".to_string(),
            Self::Equal => "=".to_string(),
            Self::Clear => "C".to_string(),
            Self::Backspace => "⌫".to_string(),
            Self::Modulus => "%".to_string(),
            Self::StartGroup => "(".to_string(),
            Self::EndGroup => ")".to_string(),
            Self::Pi => format!("*{}", std::f32::consts::PI.to_string()),
            Self::Root => format!("{} ", "math::sqrt"),
            Self::Square => "^2".to_string(),
            Self::Percentage => "/ 100.0".to_string(),
        }
    }
}
