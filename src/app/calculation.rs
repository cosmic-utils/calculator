use evalexpr::*;
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
        self.expression.push_str(&format!("{:.2}", number));
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
        let value = eval(&self.expression)?;
        log::info!("Expression -> {} = {:#?}", self.expression, value);
        self.result = match value {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => {
                if v.is_infinite() {
                    return Err(Box::new(crate::error::Error::DivisionByZero));
                } else {
                    v.to_string()
                }
            }
            _ => String::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_calculation() {
        let calc = Calculation::new();
        assert_eq!(calc.display, "");
        assert_eq!(calc.expression, "");
        assert_eq!(calc.result, "");
    }

    #[test]
    fn test_basic_addition() {
        let mut calc = Calculation::new();
        calc.on_number_press(5.0);
        calc.on_operator_press(&Operator::Add);
        calc.on_number_press(3.0);
        calc.on_equals_press().unwrap();
        assert_eq!(calc.result, "8");
    }

    #[test]
    fn test_basic_subtraction() {
        let mut calc = Calculation::new();
        calc.on_number_press(10.0);
        calc.on_operator_press(&Operator::Subtract);
        calc.on_number_press(4.0);
        calc.on_equals_press().unwrap();
        assert_eq!(calc.result, "6");
    }

    #[test]
    fn test_basic_multiplication() {
        let mut calc = Calculation::new();
        calc.on_number_press(6.0);
        calc.on_operator_press(&Operator::Multiply);
        calc.on_number_press(7.0);
        calc.on_equals_press().unwrap();
        assert_eq!(calc.result, "42");
    }

    #[test]
    fn test_basic_division() {
        let mut calc = Calculation::new();
        calc.on_number_press(15.0);
        calc.on_operator_press(&Operator::Divide);
        calc.on_number_press(3.0);
        calc.on_equals_press().unwrap();
        assert_eq!(calc.result, "5");
    }

    #[test]
    fn test_modulus() {
        let mut calc = Calculation::new();
        calc.on_number_press(17.0);
        calc.on_operator_press(&Operator::Modulus);
        calc.on_number_press(5.0);
        calc.on_equals_press().unwrap();
        assert_eq!(calc.result, "2");
    }

    #[test]
    fn test_decimal_calculation() {
        let mut calc = Calculation::new();
        calc.on_number_press(3.5);
        calc.on_operator_press(&Operator::Multiply);
        calc.on_number_press(2.0);
        calc.on_equals_press().unwrap();
        assert_eq!(calc.result, "7");
    }

    #[test]
    fn test_clear() {
        let mut calc = Calculation::new();
        calc.on_number_press(5.0);
        calc.on_operator_press(&Operator::Add);
        calc.on_number_press(3.0);
        calc.on_operator_press(&Operator::Clear);
        assert_eq!(calc.display, "");
        assert_eq!(calc.expression, "");
        assert_eq!(calc.result, "");
    }

    #[test]
    fn test_multiple_operations() {
        let mut calc = Calculation::new();
        calc.on_number_press(2.0);
        calc.on_operator_press(&Operator::Add);
        calc.on_number_press(3.0);
        calc.on_operator_press(&Operator::Multiply);
        calc.on_number_press(4.0);
        calc.on_equals_press().unwrap();
        log::info!("{}", calc.expression);
        assert_eq!(calc.result, "14");
    }

    #[test]
    fn test_division_by_zero() {
        let mut calc = Calculation::new();
        calc.on_number_press(5.0);
        calc.on_operator_press(&Operator::Divide);
        calc.on_number_press(0.0);
        assert!(calc.on_equals_press().is_err());
    }

    #[test]
    fn test_input_validation() {
        let mut calc = Calculation::new();

        // Valid input
        calc.on_input("123+456".to_string());
        assert_eq!(calc.expression, "123+456");

        // Invalid input (letters)
        calc.on_input("abc".to_string());
        assert_eq!(calc.expression, "123+456");
    }
}
