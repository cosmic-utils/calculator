use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io, process::Stdio, sync::LazyLock};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

use crate::app::operator::Operator;

static REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("\\x1B\\[(?:;?[0-9]{1,3})+[mGK]").expect("bad regex for qalc"));

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
                || c == '÷'
                || c == '%'
                || c == '.'
                || c == '\u{8}'
        }) {
            self.expression = input;
        }
    }
}

pub async fn evaluate(expression: &str, decimal_comma: bool) -> Option<String> {
    let mut command = Command::new("qalc");

    command.args(["-u8"]);
    command.args(["-set", "maxdeci 9"]);

    if decimal_comma {
        command.args(["-set", "decimal comma on"]);
    } else {
        command.args(["-set", "decimal comma off"]);
    }

    command.args(["-set", "autocalc on"]);

    let spawn = command
        .env("LANG", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let mut child = match spawn {
        Ok(child) => child,
        Err(why) => {
            return Some(if why.kind() == io::ErrorKind::NotFound {
                String::from("qalc command is not installed")
            } else {
                format!("qalc command failed to spawn: {}", why)
            });
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin
            .write_all([expression, "\n"].concat().as_bytes())
            .await;
    }

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            return Some(String::from(
                "qalc lacks stdout pipe: did you get hit by a cosmic ray?",
            ));
        }
    };

    let mut reader = BufReader::new(stdout).lines();
    let mut output = String::new();

    let _ = reader.next_line().await;
    let _ = reader.next_line().await;

    fn has_issue(line: &str) -> bool {
        line.starts_with("error") || line.starts_with("warning")
    }

    while let Ok(Some(line)) = reader.next_line().await {
        let line = line.trim();

        if line.is_empty() {
            break;
        }

        let normalized = REGEX.replace_all(line, "");
        let mut normalized = normalized.as_ref();

        if has_issue(normalized) {
            return None;
        } else {
            if !output.is_empty() {
                output.push(' ');
            }

            if normalized.starts_with('(') {
                let mut level = 1;
                for (byte_pos, character) in normalized[1..].char_indices() {
                    if character == '(' {
                        level += 1;
                    } else if character == ')' {
                        level -= 1;

                        if level == 0 {
                            normalized = normalized[byte_pos + 2..].trim_start();
                            break;
                        }
                    }
                }
            }

            let cut = if let Some(pos) = normalized.rfind('≈') {
                pos
            } else if let Some(pos) = normalized.rfind('=') {
                pos + 1
            } else {
                return None;
            };

            normalized = normalized[cut..].trim_start();
            if normalized.starts_with('(') && normalized.ends_with(')') {
                normalized = &normalized[1..normalized.len() - 1];
            }

            output.push_str(&normalized.replace('\u{2212}', "-"));
        };
    }

    Some(output)
}

/// Checks if the system uses a decimal comma instead of a decimal point.
pub async fn uses_decimal_comma() -> bool {
    let spawn_result = Command::new("locale")
        .arg("-ck")
        .arg("decimal_point")
        .stderr(Stdio::null())
        .output()
        .await;

    if let Ok(output) = spawn_result
        && let Ok(string) = String::from_utf8(output.stdout)
    {
        return string.contains("decimal_point=\",\"");
    }

    false
}

/// Extracts the value from an outcome expression.
pub fn extract_value(expression: &str) -> &str {
    expression
        .rfind('=')
        .map(|p| p + 1)
        .or_else(|| expression.rfind('≈').map(|p| p + 3))
        .map(|pos| expression[pos..].trim())
        .unwrap_or(expression)
}
