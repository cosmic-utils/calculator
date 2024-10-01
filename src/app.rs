// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::collections::HashMap;

use crate::calculation::Calculation;
use crate::config;
use crate::config::CONFIG_VERSION;
use crate::fl;
use crate::operator::Operator;
use cosmic::app::{self, Command, Core, Message as CosmicMessage};
use cosmic::cosmic_config::Update;
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::{
    event,
    keyboard::Event as KeyEvent,
    keyboard::{Key, Modifiers},
    Alignment, Event, Length, Subscription,
};
use cosmic::widget::menu::Action;
use cosmic::widget::{self, menu, nav_bar};
use cosmic::{cosmic_config, cosmic_theme, theme, Application, ApplicationExt, Element};
const REPOSITORY: &str = "https://github.com/cosmic-utils/calculator";

pub struct Calculator {
    core: Core,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    nav: nav_bar::Model,
    modifiers: Modifiers,
    config_handler: Option<cosmic_config::Config>,
    config: config::CalculatorConfig,
    calculation: Calculation,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    Number(f32),
    Operator(Operator),
    Input(String),
    ToggleContextPage(ContextPage),
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeModeChange,
    NavMenuAction(NavMenuAction),
    CleanHistory,
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

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: config::CalculatorConfig,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    ClearHistory,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::ClearHistory => Message::CleanHistory,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavMenuAction {
    Delete(nav_bar::Id),
}

impl menu::action::MenuAction for NavMenuAction {
    type Message = cosmic::app::Message<Message>;

    fn message(&self) -> Self::Message {
        cosmic::app::Message::App(Message::NavMenuAction(*self))
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

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        self.nav.activate(id);
        self.nav
            .active_data()
            .map_or(Command::none(), |data: &Calculation| {
                self.calculation.expression = data.result.to_string().clone();
                self.calculation.result = 0.0;
                self.calculation.display = data.expression.to_string();
                Command::none()
            })
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        for entry in &flags.config.history {
            nav.insert()
                .text(entry.to_string().clone())
                .data(entry.clone());
        }

        let mut app = Calculator {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            nav,
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            calculation: Calculation::new(),
        };

        let set_window_title = app.set_window_title(fl!("app-title"));

        (app, set_window_title)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("clear-history"), MenuAction::ClearHistory),
                    menu::Item::Button(fl!("about"), MenuAction::About),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn nav_context_menu(
        &self,
        id: nav_bar::Id,
    ) -> Option<Vec<menu::Tree<CosmicMessage<Self::Message>>>> {
        Some(cosmic::widget::menu::items(
            &HashMap::new(),
            vec![cosmic::widget::menu::Item::Button(
                fl!("delete"),
                NavMenuAction::Delete(id),
            )],
        ))
    }

    fn view(&self) -> Element<Self::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        widget::column::with_capacity(2)
            .push(
                widget::text_input("", &self.calculation.display)
                    .on_input(Message::Input)
                    .on_submit(Message::Operator(Operator::Equal))
                    .size(32.0)
                    .width(Length::Fill),
            )
            .push(
                widget::column::with_capacity(5)
                    .push(
                        widget::row::with_capacity(3)
                            .push(
                                widget::row::with_capacity(1)
                                    .push(wide_button(Message::Operator(Operator::Clear), 2)),
                            )
                            .push(
                                widget::row::with_capacity(2)
                                    .push(standard_button(Message::Operator(Operator::Modulus)))
                                    .push(suggested_button(Message::Operator(Operator::Divide)))
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .spacing(spacing.space_xs),
                            )
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(Message::Number(7.0)))
                            .push(standard_button(Message::Number(8.0)))
                            .push(standard_button(Message::Number(9.0)))
                            .push(suggested_button(Message::Operator(Operator::Multiply)))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(Message::Number(4.0)))
                            .push(standard_button(Message::Number(5.0)))
                            .push(standard_button(Message::Number(6.0)))
                            .push(suggested_button(Message::Operator(Operator::Subtract)))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(Message::Number(1.0)))
                            .push(standard_button(Message::Number(2.0)))
                            .push(standard_button(Message::Number(3.0)))
                            .push(suggested_button(Message::Operator(Operator::Add)))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(Message::Number(0.0)))
                            .push(standard_button(Message::Operator(Operator::Point)))
                            .push(standard_button(Message::Operator(Operator::Backspace)))
                            .push(suggested_button(Message::Operator(Operator::Equal)))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .max_width(1000.0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .spacing(spacing.space_xs),
            )
            .align_items(Alignment::Center)
            .spacing(spacing.space_s)
            .padding(spacing.space_xxs)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
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
            Message::Input(input) => self.calculation.on_input(input),
            Message::Operator(operator) => {
                self.calculation.on_operator_press(&operator);
                if operator == Operator::Equal {
                    let mut history = self.config.history.clone();
                    history.push(self.calculation.clone());
                    config_set!(history, history);
                    self.nav
                        .insert()
                        .text(self.calculation.to_string())
                        .data(self.calculation.clone());
                    self.calculation.display = self.calculation.result.to_string();
                }
            }
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
            Message::NavMenuAction(action) => match action {
                NavMenuAction::Delete(entity) => {
                    if let Some(data) = self
                        .nav
                        .data::<Calculation>(entity)
                        .map(|data| data.clone())
                    {
                        let mut history = self.config.history.clone();
                        history.retain(|calc| calc != &data);
                        config_set!(history, history);
                        self.nav.remove(entity);
                    }
                }
            },
            Message::SystemThemeModeChange => {
                return self.update_config();
            }
            Message::CleanHistory => {
                config_set!(history, vec![]);
                self.nav.clear();
            }
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

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;

        let subscriptions = vec![
            event::listen_with(|event, status| match event {
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

    fn update_config(&mut self) -> Command<Message> {
        app::command::set_theme(self.config.app_theme.theme())
    }
}

pub fn wide_button<'a>(message: Message, portion: u16) -> Element<'a, Message> {
    let label = match message.clone() {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.display().to_string(),
        _ => String::new(),
    };
    button(
        label,
        message,
        theme::Button::Standard,
        Length::FillPortion(portion),
    )
}

pub fn standard_button<'a>(message: Message) -> Element<'a, Message> {
    let label = match message.clone() {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.display().to_string(),
        _ => String::new(),
    };
    button(label, message, theme::Button::Standard, Length::Fill)
}

pub fn suggested_button<'a>(message: Message) -> Element<'a, Message> {
    let label = match &message {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.display().to_string(),
        _ => String::new(),
    };
    button(
        label.to_string(),
        message,
        theme::Button::Suggested,
        Length::Fill,
    )
}

pub fn button<'a>(
    label: String,
    message: Message,
    style: theme::Button,
    width: Length,
) -> Element<'a, Message> {
    widget::button(
        widget::container(widget::text(label).size(20.0))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .style(style)
    .width(width)
    .height(Length::Fill)
    .on_press(message)
    .into()
}
