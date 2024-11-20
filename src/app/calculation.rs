use calculator_rs::{Calculate, Value};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt::Display};

use crate::app::operator::Operator;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Calculation {
    pub display: String,
    pub expression: String,
    pub result: String,
}

impl Display for Calculation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.display, self.result)
    }
}

pub enum Message {
    Continue,
    Error(String),
}

impl Calculation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_operator(&mut self, operator: Operator) {
        self.expression.push_str(operator.expression());
        self.display.push_str(operator.display());
    }

    pub fn on_number_press(&mut self, number: f32) {
        self.display.push_str(&number.to_string());
        self.expression.push_str(&number.to_string());
    }

    pub fn on_operator_press(&mut self, operator: &Operator) -> Message {
        match operator {
            Operator::Add => self.add_operator(Operator::Add),
            Operator::Subtract => self.add_operator(Operator::Subtract),
            Operator::Multiply => self.add_operator(Operator::Multiply),
            Operator::Divide => self.add_operator(Operator::Divide),
            Operator::Modulus => self.add_operator(Operator::Modulus),
            Operator::Point => self.add_operator(Operator::Point),
            Operator::Clear => self.clear(),
            Operator::Equal => {
                if let Err(err) = self.on_equals_press() {
                    log::error!("{err}");
                    return Message::Error(err.to_string());
                }
            }
            Operator::Backspace => {
                self.expression.pop();
            }
        };
        Message::Continue
    }

    pub fn on_equals_press(&mut self) -> Result<(), Box<dyn Error>> {
        log::info!("Expression -> {}", self.expression);
        self.result = match self.expression.calculate()? {
            Value::Integer(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
        };
        Ok(())
    }

    pub fn clear(&mut self) {
        self.display.clear();
        self.expression.clear();
        self.result = String::new();
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
