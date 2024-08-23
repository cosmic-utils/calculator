use calculator_rs::{Calculate, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::operator::Operator;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Calculation {
    pub display: String,
    pub expression: String,
    pub result: f64,
}

impl Display for Calculation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.display, self.result)
    }
}

impl Calculation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_operator(&mut self, operator: Operator) {
        self.expression.push_str(&operator.expression());
        self.display.push_str(&operator.display());
    }

    pub fn on_number_press(&mut self, number: f32) {
        self.display.push_str(&number.to_string());
        self.expression.push_str(&format!("{:.1}", number));
    }

    pub fn on_operator_press(&mut self, operator: &Operator) {
        match operator {
            Operator::Add => self.add_operator(Operator::Add),
            Operator::Subtract => self.add_operator(Operator::Subtract),
            Operator::Multiply => self.add_operator(Operator::Multiply),
            Operator::Divide => self.add_operator(Operator::Divide),
            Operator::Modulus => self.add_operator(Operator::Modulus),
            Operator::Point => self.add_operator(Operator::Point),
            Operator::Clear => self.clear(),
            Operator::Equal => self.on_equals_press(),
            Operator::Backspace => {
                self.expression.pop();
            }
        }
    }

    pub fn on_equals_press(&mut self) {
        let calculation = self.expression.calculate();
        self.result = match calculation {
            Ok(value) => match value {
                Value::Integer(v) => v as f64,
                Value::Float(v) => v,
            },
            Err(_) => 0.0,
        };
    }

    pub fn clear(&mut self) {
        self.display.clear();
        self.expression.clear();
        self.result = 0.0;
    }

    pub(crate) fn on_input(&mut self, input: String) {
        if input.chars().all(|c| {
            c.is_digit(10)
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
