use serde::{Deserialize, Serialize};

use crate::app::operator::Operator;

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Calculation {
    pub expression: String,
    pub result: String,
}

pub enum Message {
    Calculate(String),
}

impl Calculation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_operator(&mut self, operator: Operator) {
        self.expression.push_str(operator.expression());
    }

    pub fn on_number_press(&mut self, number: i32) {
        self.expression.push_str(&format!("{}", number));
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
            Operator::Equal => return Some(Message::Calculate(self.expression.clone())),
            Operator::Backspace => {
                self.expression.pop();
            }
        };
        None
    }

    pub fn clear(&mut self) {
        self.expression.clear();
        self.result.clear();
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
