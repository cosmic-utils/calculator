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
    #[serde(skip)]
    pub outcome_exact: String,
    #[serde(skip)]
    pub chain_exact: bool,
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
        self.expression.push_str(operator.expression(self.decimal_comma));
    }

    pub fn on_number_press(&mut self, number: f32) {
        self.expression.push_str(&number.to_string());
    }

    /// Expression to evaluate: the exact form of the last result replaces the
    /// displayed one when it is the untouched prefix and an operator follows.
    pub fn eval_expression(&self) -> String {
        let expression = self.expression.trim();
        let chainable =
            self.chain_exact && !self.outcome.is_empty() && !self.outcome_exact.is_empty();
        match expression.strip_prefix(self.outcome.as_str()) {
            Some(tail)
                if chainable
                    && tail.starts_with(['+', '-', '*', '/', '×', '÷', '%', '^', '!']) =>
            {
                format!("({}){}", self.outcome_exact, tail)
            }
            _ => expression.to_string(),
        }
    }

    /// Builds the qalc command: exact output for chaining, 18 decimals for display.
    pub fn qalc_command(&self, exact: bool) -> Command {
        let mut command = Command::new("qalc");
        // Never let qalc block waiting on stdin.
        command.stdin(Stdio::null());
        command.env("LANG", "C");
        command.args(["-t", "-u8", "-set", "precision 50"]);
        if exact {
            command.args(["-set", "fr exact", "-set", "maxdeci off"]);
            command.args(["-set", "unicode off"]);
        } else {
            command.args(["-set", "maxdeci 18"]);
        }
        if self.decimal_comma {
            command.args(["-set", "decimal comma on"]);
        } else {
            command.args(["-set", "decimal comma off"]);
        }
        if autocalc() {
            command.args(["-set", "autocalc off"]);
        }
        command
    }

    pub fn on_operator_press(&mut self, operator: &Operator) -> Option<Message> {
        match operator {
            Operator::Add
            | Operator::Subtract
            | Operator::Multiply
            | Operator::Divide
            | Operator::Modulus
            | Operator::Point
            | Operator::ParenthesesOpen
            | Operator::ParenthesesClose
            | Operator::Power
            | Operator::SquareRoot
            | Operator::Comma
            | Operator::Log
            | Operator::Ln
            | Operator::Log2
            | Operator::Factorial
            | Operator::Sin
            | Operator::Cos
            | Operator::Tan
            | Operator::Asin 
            | Operator::Acos 
            | Operator::Atan
            | Operator::Pi
            | Operator::E
            | Operator::Reciprocal
            | Operator::Sinh
            | Operator::Cosh
            | Operator::Tanh
            | Operator::Degree
            | Operator::Square
            | Operator::Cube
            | Operator::PowerOfTen
            | Operator::Exp
            | Operator::Abs
            | Operator::CubeRoot
            | Operator::Sum
            | Operator::Mean
            | Operator::Median
            | Operator::Mode
            | Operator::Min
            | Operator::Max
            | Operator::StdDev
            | Operator::StdDevPopulation
            | Operator::Variance
            | Operator::VariancePopulation
            | Operator::Combinations
            | Operator::Permutations => self.add_operator(operator.clone()),

            Operator::Clear => self.clear(),
            Operator::Negate => self.toggle_sign(),
            Operator::Equal => return Some(Message::Evaluate),
            Operator::Backspace => {
                self.expression.pop();
            }
        };
        None
    }
    pub fn toggle_sign(&mut self) {
        // Start index of the trailing number, if the expression ends with one.
        let Some(num_start) = self
            .expression
            .char_indices()
            .rev()
            .take_while(|(_, c)| c.is_ascii_digit() || *c == '.' || *c == ',')
            .last()
            .map(|(i, _)| i)
        else {
            return; // empty or not ending in a number: nothing to negate
        };

        let before = &self.expression[..num_start];
        // A '-' is unary at the start or right after an operator or '('.
        let is_unary_minus = before.ends_with('-')
            && matches!(
                before[..before.len() - 1].chars().next_back(),
                None | Some('+' | '-' | '*' | '/' | '×' | '÷' | '%' | '^' | '(')
            );

        if is_unary_minus {
            self.expression.remove(num_start - 1);
        } else {
            self.expression.insert(num_start, '-');
        }
    }

    pub fn clear(&mut self) {
        self.expression.clear();
        self.outcome = String::new();
        self.outcome_exact = String::new();
        self.chain_exact = false;
    }

    pub(crate) fn on_input(&mut self, input: String) {
        // Exact chaining stays valid only while the result prefix is untouched.
        self.chain_exact =
            self.chain_exact && !self.outcome.is_empty() && input.starts_with(&self.outcome);
        // qalc validates the expression itself, so keep this filter permissive:
        // allow letters (sin, pi), whitespace, '!', and ',' for decimal-comma locales.
        if input.chars().all(|c| {
            c.is_alphanumeric()
                || c.is_whitespace()
                || matches!(
                    c,
                    '+' | '-'
                        | '*'
                        | '/'
                        | '÷'
                        | '×'
                        | '%'
                        | '.'
                        | ','
                        | ';'
                        | '('
                        | ')'
                        | '^'
                        | '√'
                        | '⁻'
                        | '°'
                        | '!'
                        | '\u{8}'
                )
        }) {
            self.expression = input;
        }
    }
}

/// Cleans qalc stdout into one line, stripping any interactive-prompt artifact.
pub fn parse_output(stdout: Vec<u8>) -> String {
    String::from_utf8(stdout)
        .unwrap_or_default()
        .replace(['\n', '\r'], "")
        .replace("> ", "")
        .trim()
        .to_string()
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
        .is_some_and(|current| current >= min_version)
}