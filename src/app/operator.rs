#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    DecimalSeparator,
    Equal,
    Clear,
    Backspace,
    Negate,
    ParenthesesOpen,
    ParenthesesClose,
    Exponent,
    SquareRoot,
    // Scientific
    Comma,
    Log,
    Ln,
    Log2,
    Factorial,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Pi,
    E,
    Reciprocal,
}

impl Operator {
    pub fn display(&self, decimal_comma: bool) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "−", // Unicode Minus Sign
            Self::Multiply => "×", // Unicode Multiplication Sign
            Self::Divide => "÷",   // Unicode Division Sign
            Self::Modulus => "%",
            Self::DecimalSeparator => if decimal_comma {","} else {"."},
            Self::Equal => "=",
            Self::ParenthesesOpen => "(",
            Self::ParenthesesClose => ")",
            Self::Exponent => "xʸ",
            Self::SquareRoot => "√",
            Self::Clear => "C",
            Self::Backspace => "⌫",
            Self::Negate => "±",
            // Scientific
            Self::Comma => if decimal_comma {";"} else {","},
            Self::Log => "log",
            Self::Ln => "ln",
            Self::Log2 => "log2",
            Self::Factorial => "!",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Pi => "π",
            Self::E => "e",
            Self::Reciprocal => "1/x",
        }
    }

    pub fn expression(&self, decimal_comma: bool) -> &str {
        match self {
            Self::Add => "+",
            Self::Subtract => "−", // Unicode Minus Sign
            Self::Multiply => "×", // Unicode Multiplication Sign
            Self::Divide => "÷",   // Unicode Division Sign
            Self::Modulus => "%",
            Self::DecimalSeparator => if decimal_comma {","} else {"."},
            Self::Equal => "=",
            Self::ParenthesesOpen => "(",
            Self::ParenthesesClose => ")",
            Self::Exponent => "^",
            Self::SquareRoot => "√",
            Self::Clear => "C",
            Self::Backspace => "⌫",
            Self::Negate => "±",
            Self::Comma => if decimal_comma {";"} else {","},
            Self::Log => "log",
            Self::Ln => "ln",
            Self::Log2 => "log2(",
            Self::Factorial => "!",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Pi => "π",
            Self::E => "e",
            Self::Reciprocal => "⁻¹",
        }
    }
}
