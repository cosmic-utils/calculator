use calculator_rs::{Calculate, Value};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::operator::Operator;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Calculation(pub String, pub f64);

impl Display for Calculation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} = {}", self.0, self.1)
    }
}

impl Calculation {
    pub fn new() -> Self {
        Self(String::new(), 0.0)
    }

    pub fn on_number_press(&mut self, number: i8) {
        self.0.push_str(&number.to_string());
    }

    pub fn on_operator_press(&mut self, operator: &Operator) {
        match operator {
            Operator::Add => self.0.push_str(&Operator::Add.to_string()),
            Operator::Subtract => self.0.push_str(&Operator::Subtract.to_string()),
            Operator::Multiply => self.0.push_str(&Operator::Multiply.to_string()),
            Operator::Divide => self.0.push_str(&Operator::Divide.to_string()),
            Operator::Modulus => self.0.push_str(&Operator::Modulus.to_string()),
            Operator::Clear => self.clear(),
            Operator::ClearEntry => self.clear_entry(),
            Operator::Point => self.0.push_str("."),
            Operator::Backspace => {
                self.0.pop();
            }
            Operator::Equal => self.on_equals_press(),
        }
    }

    pub fn on_equals_press(&mut self) {
        self.1 = match self.0.calculate() {
            Ok(value) => match value {
                Value::Integer(v) => v as f64,
                Value::Float(v) => v,
            },
            Err(_) => 0.0,
        };
    }

    pub fn clear(&mut self) {
        self.0.clear();
        self.1 = 0.0;
    }

    fn clear_entry(&mut self) {
        self.0.clear();
    }
}
