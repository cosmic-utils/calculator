// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::collections::HashMap;

use crate::app::{calculation::Calculation, config::CONFIG_VERSION, operator::Operator};
use crate::core::key_binds::key_binds;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::app::{self, Core, Task};
use cosmic::cosmic_config::Update;
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::{
    event,
    keyboard::Event as KeyEvent,
    keyboard::{Key, Modifiers},
    Alignment, Event, Length, Subscription,
};
use cosmic::widget::about::About;
use cosmic::widget::menu::Action;
use cosmic::widget::{self, container, dropdown, menu, ToastId};
use cosmic::{cosmic_config, cosmic_theme, theme, Application, ApplicationExt, Element};

mod calculation;
mod config;
mod operator;
pub mod settings;

pub struct Calculator {
    core: Core,
    about: About,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    modifiers: Modifiers,
    config_handler: Option<cosmic_config::Config>,
    config: config::CalculatorConfig,
    calculation: Calculation,
    toasts: widget::Toasts<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Number(u8),
    Operator(Operator),
    Input(String),
    ToggleContextPage(ContextPage),
    ToggleContextDrawer,
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeModeChange,
    DropdownAction(usize),
    CleanHistory,
    Undo,
    ShowToast(String),
    CloseToast(ToastId),
    Open(String),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: config::CalculatorConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    ClearHistory,
    Undo,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::ClearHistory => Message::CleanHistory,
            MenuAction::Undo => Message::Undo,
        }
    }
}

impl Application for Calculator {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = Message;

    const APP_ID: &'static str = "dev.edfloreshz.Calculator";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // let mut nav = nav_bar::Model::default();

        // for entry in &flags.config.history {
        //     nav.insert()
        //         .text(entry.to_string().clone())
        //         .data(entry.clone());
        // }

        let about = About::default()
            .name(fl!("app-title"))
            .icon(Self::APP_ID)
            .version("0.1.1")
            .author("Eduardo Flores")
            .license("GPL-3.0-only")
            .links([
                (
                    fl!("support"),
                    "https://github.com/cosmic-utils/calculator/issues",
                ),
                (
                    fl!("repository"),
                    "https://github.com/cosmic-utils/calculator",
                ),
            ])
            .developers([("Eduardo Flores", "edfloreshz@proton.me")]);

        let mut app = Calculator {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            calculation: Calculation::new(),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
        };

        let mut tasks = vec![];

        tasks.push(app.set_window_title(fl!("app-title")));

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        // let button = Button::new("Undo").class(theme::Button::Standard);
        let but = widget::button::custom(
            widget::container(widget::text("Undo"))
                .center(Length::Fill)
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .class(theme::Button::HeaderBar)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .on_press(Message::Undo);

        vec![but.into()]
    }

    fn header_center(&self) -> Vec<Element<Self::Message>> {
        let dropdown = dropdown(&["Basic", "Programming"], Some(0), Message::DropdownAction);
        vec![dropdown.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        container(
            widget::column::with_capacity(3)
                .push(
                    widget::row::row()
                        .push(widget::toaster(&self.toasts, widget::vertical_space())),
                )
                .push(
                    widget::text_input("", &self.calculation.display)
                        .on_input(Message::Input)
                        .on_submit(Message::Operator(Operator::Equal))
                        .size(32.0)
                        .width(Length::Fill),
                )
                .push(
                    widget::row::with_capacity(5)
                        .push(
                            widget::column::with_capacity(5)
                                .push(standard_button(
                                    Message::Operator(Operator::Clear),
                                    Length::FillPortion(1),
                                ))
                                .push(suggested_button(Message::Number(7), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(4), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(1), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(0), Length::FillPortion(1)))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .spacing(spacing.space_xxxs),
                        )
                        .push(
                            widget::column::with_capacity(5)
                                .push(standard_button(
                                    Message::Operator(Operator::StartGroup),
                                    Length::FillPortion(1),
                                ))
                                .push(suggested_button(Message::Number(8), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(5), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(2), Length::FillPortion(1)))
                                .push(standard_button(
                                    Message::Operator(Operator::Point),
                                    Length::FillPortion(1),
                                ))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .spacing(spacing.space_xxxs),
                        )
                        .push(
                            widget::column::with_capacity(5)
                                .push(standard_button(
                                    Message::Operator(Operator::EndGroup),
                                    Length::FillPortion(1),
                                ))
                                .push(suggested_button(Message::Number(9), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(6), Length::FillPortion(1)))
                                .push(suggested_button(Message::Number(3), Length::FillPortion(1)))
                                .push(standard_button(
                                    Message::Operator(Operator::Percentage),
                                    Length::FillPortion(1),
                                ))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .spacing(spacing.space_xxxs),
                        )
                        .push(
                            widget::column::with_capacity(5)
                                .push(standard_button(
                                    Message::Operator(Operator::Modulus),
                                    Length::FillPortion(1),
                                ))
                                .push(standard_button(
                                    Message::Operator(Operator::Divide),
                                    Length::FillPortion(1),
                                ))
                                .push(standard_button(
                                    Message::Operator(Operator::Multiply),
                                    Length::FillPortion(1),
                                ))
                                .push(standard_button(
                                    Message::Operator(Operator::Subtract),
                                    Length::FillPortion(1),
                                ))
                                .push(standard_button(
                                    Message::Operator(Operator::Add),
                                    Length::FillPortion(1),
                                ))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .spacing(spacing.space_xxxs),
                        )
                        .push(
                            widget::column::with_capacity(5)
                                .push(standard_button(
                                    Message::Operator(Operator::Pi),
                                    Length::FillPortion(1),
                                ))
                                .push(standard_button(
                                    Message::Operator(Operator::Root),
                                    Length::FillPortion(1),
                                ))
                                .push(standard_button(
                                    Message::Operator(Operator::Square),
                                    Length::FillPortion(1),
                                ))
                                .push(suggested_button(
                                    Message::Operator(Operator::Equal),
                                    Length::FillPortion(2),
                                ))
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .spacing(spacing.space_xxxs),
                        )
                        .width(Length::Fill)
                        .height(Length::Fixed(200.0))
                        .align_y(Alignment::Center)
                        .spacing(spacing.space_none),
                )
                .align_x(Alignment::Center)
                .spacing(spacing.space_l)
                .padding(spacing.space_xxs)
                .width(Length::Fixed(700.0)), // .into()
        )
        .center(Length::Fill)
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let mut commands = vec![];

        // Helper for updating config values efficiently
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        match message {
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    log::error!("{err}")
                }
            }
            Message::ShowToast(message) => {
                commands.push(
                    self.toasts
                        .push(widget::toaster::Toast::new(message))
                        .map(cosmic::app::Message::App),
                );
            }
            Message::CloseToast(id) => self.toasts.remove(id),
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            Message::ToggleContextDrawer => {
                self.core.window.show_context = !self.core.window.show_context;
            }
            Message::Number(num) => self.calculation.on_number_press(num),
            Message::Input(input) => self.calculation.on_input(input),
            Message::Operator(operator) => match self.calculation.on_operator_press(&operator) {
                crate::app::calculation::Message::Continue => {
                    if operator == Operator::Equal {
                        let mut history = self.config.history.clone();
                        history.push(self.calculation.clone());
                        config_set!(history, history);
                        self.calculation.display = self.calculation.result.to_string();
                    }
                }
                crate::app::calculation::Message::Error(message) => {
                    let command = self.update(Message::ShowToast(message));
                    commands.push(command);
                }
            },
            Message::Key(modifiers, key) => {
                for (key_bind, action) in &self.key_binds {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::DropdownAction(id) => {}
            Message::SystemThemeModeChange => {
                return self.update_config();
            }
            Message::CleanHistory => {
                config_set!(history, vec![]);
            }
            Message::Undo => {
                let mut last = self.config.history.clone();
                println!("{:?}", last);
                let prev = last.pop();
                config_set!(history, last);
                if let Some(v) = prev {
                    self.calculation.display = v.expression;
                }
            }
        }
        Task::batch(commands)
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => {
                context_drawer::about(&self.about, Message::Open, Message::ToggleContextDrawer)
            }
        })
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let subscriptions = vec![
            event::listen_with(|event, status, _id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update: Update<ThemeMode>| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange
            }),
        ];

        Subscription::batch(subscriptions)
    }
}

impl Calculator {
    fn update_config(&mut self) -> Task<Message> {
        app::command::set_theme(self.config.app_theme.theme())
    }
}

pub fn standard_button<'a>(message: Message, height: Length) -> Element<'a, Message> {
    let label = match message.clone() {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.button_display().to_string(),
        _ => String::new(),
    };
    button(label, message, theme::Button::Standard, height)
}

pub fn suggested_button<'a>(message: Message, height: Length) -> Element<'a, Message> {
    let label = match &message {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.button_display().to_string(),
        _ => String::new(),
    };
    button(label.to_string(), message, theme::Button::Suggested, height)
}

pub fn button<'a>(
    label: String,
    message: Message,
    style: theme::Button,
    height: Length,
) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label).size(20.0))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .class(style)
    .width(Length::Fill)
    .height(height)
    .on_press(message)
    .into()
}
