use crate::app::operator::Operator;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::{Command, Stdio},
};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct Calculator {
    pub expression: String,
    pub outcome: String,
    pub decimal_comma: bool,
}

impl Display for Calculator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.outcome)
    }
}

pub enum Message {
    Evaluate,
}

impl Calculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_operator(&mut self, operator: Operator) {
        self.expression.push_str(operator.expression());
    }

    pub fn on_number_press(&mut self, number: f32) {
        self.expression.push_str(&number.to_string());
    }

    pub fn on_operator_press(&mut self, operator: &Operator) -> Option<Message> {
        match operator {
            Operator::Add => self.add_operator(Operator::Add),
            Operator::Subtract => self.add_operator(Operator::Subtract),
            Operator::Multiply => self.add_operator(Operator::Multiply),
            Operator::Divide => self.add_operator(Operator::Divide),
            Operator::Modulus => self.add_operator(Operator::Modulus),
            Operator::Point => self.add_operator(Operator::Point),
            Operator::ParenthesesOpen => self.add_operator(Operator::ParenthesesOpen),
            Operator::ParenthesesClose => self.add_operator(Operator::ParenthesesClose),
            Operator::Power => self.add_operator(Operator::Power),
            Operator::SquareRoot => self.add_operator(Operator::SquareRoot),
            Operator::Clear => self.clear(),
            Operator::Equal => return Some(Message::Evaluate),
            Operator::Backspace => {
                self.expression.pop();
            }
        };
        None
    }
    pub fn clear(&mut self) {
        self.expression.clear();
        self.outcome = String::new();
    }

    pub(crate) fn on_input(&mut self, input: String) {
        if input.chars().all(|c| {
            c.is_ascii_digit()
                || c == '+'
                || c == '-'
                || c == '*'
                || c == '/'
                || c == '÷'
                || c == '%'
                || c == '.'
                || c == '\u{8}'
        }) {
            self.expression = input;
        }
    }
}

/// Returns the version of the `qalc` command-line tool.
fn qalc_version() -> Option<String> {
    let output = Command::new("qalc").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(version)
}

/// Checks if the system uses a decimal comma instead of a decimal point.
pub async fn uses_decimal_comma() -> bool {
    let spawn_result = Command::new("locale")
        .arg("-ck")
        .arg("decimal_point")
        .stderr(Stdio::null())
        .output();

    if let Ok(output) = spawn_result
        && let Ok(string) = String::from_utf8(output.stdout)
    {
        return string.contains("decimal_point=\",\"");
    }

    false
}

pub fn autocalc() -> bool {
    let min_version = Version::parse("5.4.0").unwrap();
    qalc_version()
        .and_then(|version| Version::parse(&version).ok())
        .map_or(false, |current| current >= min_version)
}
