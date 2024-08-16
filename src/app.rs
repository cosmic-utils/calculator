// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use std::fmt::Display;

use crate::fl;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, menu};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};

const REPOSITORY: &str = "https://github.com/edfloreshz/cosmic-ext-calculator";

pub struct Calculator {
    core: Core,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    calculation: Calculation,
}

#[derive(Debug)]
pub struct Calculation {
    pub expression: String,
    pub operator: Option<Operator>,
    pub result: f64,
    pub previous_press_operator: bool,
    pub equals_pressed: bool,
}

impl Calculation {
    pub fn new() -> Self {
        Self {
            expression: String::new(),
            operator: None,
            result: 0.0,
            previous_press_operator: false,
            equals_pressed: false,
        }
    }

    pub fn reset(&mut self) {
        self.result = 0.0;
        self.expression.clear();
        self.operator = None;
        self.equals_pressed = false;
        self.previous_press_operator = false;
    }

    pub fn on_number_press(&mut self, number: i8) {
        if self.previous_press_operator {
            self.expression.clear();
            self.previous_press_operator = false;
        }
        if self.equals_pressed {
            self.reset();
        }
        self.expression.push_str(&number.to_string());
    }

    pub fn on_operator_press(&mut self, operator: Operator) {
        if self.equals_pressed {
            self.reset();
        }
        match operator {
            Operator::Clear => {
                self.reset();
            }
            Operator::Point => {
                self.expression.push_str(".");
            }
            _ => {
                self.on_equals_press();
            }
        }
        self.operator = Some(operator);
        self.previous_press_operator = true;
    }

    pub fn on_equals_press(&mut self) {
        if self.result == 0.0 {
            if let Ok(r) = self.expression.parse::<f64>() {
                self.result = r;
            };
            return;
        }

        if let Some(operator) = &self.operator {
            let result = match operator {
                Operator::Add => self.result + self.expression.parse::<f64>().unwrap(),
                Operator::Subtract => self.result - self.expression.parse::<f64>().unwrap(),
                Operator::Multiply => self.result * self.expression.parse::<f64>().unwrap(),
                Operator::Divide => self.result / self.expression.parse::<f64>().unwrap(),
                Operator::Modulus => self.result % self.expression.parse::<f64>().unwrap(),
                _ => self.result,
            };

            self.result = result;
            self.expression = self.result.to_string();
            self.operator = None;
            self.equals_pressed = true;
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    Number(i8),
    Modifier(Operator),
    ToggleContextPage(ContextPage),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Point,
    Equal,
    Clear,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let symbol = match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "×",
            Self::Divide => "÷",
            Self::Modulus => "%",
            Self::Point => ".",
            Self::Equal => "=",
            Self::Clear => "C",
        };

        write!(f, "{}", symbol)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

impl Application for Calculator {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.edfloreshz.Calculator";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app = Calculator {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            calculation: Calculation::new(),
        };

        (app, Command::none())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        widget::column::with_children(vec![
            widget::text_input("", &self.calculation.expression)
                .size(28.0)
                .width(Length::Fill)
                .into(),
            widget::grid()
                .column_spacing(16)
                .row_spacing(16)
                .push(Calculator::button("1", Message::Number(1)))
                .push(Calculator::button("2", Message::Number(2)))
                .push(Calculator::button("3", Message::Number(3)))
                .push(Calculator::button("÷", Message::Modifier(Operator::Divide)))
                .insert_row()
                .push(Calculator::button("4", Message::Number(4)))
                .push(Calculator::button("5", Message::Number(5)))
                .push(Calculator::button("6", Message::Number(6)))
                .push(Calculator::button(
                    "×",
                    Message::Modifier(Operator::Multiply),
                ))
                .insert_row()
                .push(Calculator::button("7", Message::Number(7)))
                .push(Calculator::button("8", Message::Number(8)))
                .push(Calculator::button("9", Message::Number(9)))
                .push(Calculator::button(
                    "-",
                    Message::Modifier(Operator::Subtract),
                ))
                .insert_row()
                .push(Calculator::button("0", Message::Number(0)))
                .push(Calculator::button(".", Message::Modifier(Operator::Point)))
                .push(Calculator::button(
                    "%",
                    Message::Modifier(Operator::Modulus),
                ))
                .push(Calculator::button("+", Message::Modifier(Operator::Add)))
                .insert_row()
                .push(Calculator::button("C", Message::Modifier(Operator::Clear)))
                .push(Calculator::button("=", Message::Modifier(Operator::Equal)))
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),
        ])
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }

                self.set_context_title(context_page.title());
            }
            Message::Number(num) => self.calculation.on_number_press(num),
            Message::Modifier(operator) => self.calculation.on_operator_press(operator),
        }
        Command::none()
    }

    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
        })
    }
}

impl Calculator {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/scalable/apps/dev.edfloreshz.Calculator.svg")[..],
        ));

        let title = widget::text::title3(fl!("app-title"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn button(label: &str, message: Message) -> Element<Message> {
        widget::button(
            widget::container(widget::text(label).size(20.0))
                .center_x()
                .center_y(),
        )
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0))
        .on_press(message)
        .into()
    }
}
