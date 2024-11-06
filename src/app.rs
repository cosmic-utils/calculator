// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::collections::HashMap;

use crate::calculation::Calculation;
use crate::config;
use crate::config::CONFIG_VERSION;
use crate::fl;
use crate::operator::Operator;
use cosmic::app::about::About;
use cosmic::app::{self, Core, Message as CosmicMessage, Task};
use cosmic::cosmic_config::Update;
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::{
    event,
    keyboard::Event as KeyEvent,
    keyboard::{Key, Modifiers},
    Alignment, Event, Length, Subscription,
};
use cosmic::widget::menu::Action;
use cosmic::widget::{self, menu, nav_bar, ToastId};
use cosmic::{cosmic_config, cosmic_theme, theme, Application, ApplicationExt, Element};

pub struct Calculator {
    core: Core,
    about: About,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    nav: nav_bar::Model,
    modifiers: Modifiers,
    config_handler: Option<cosmic_config::Config>,
    config: config::CalculatorConfig,
    calculation: Calculation,
    toasts: widget::Toasts<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Number(f32),
    Operator(Operator),
    Input(String),
    ToggleContextPage(ContextPage),
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeModeChange,
    NavMenuAction(NavMenuAction),
    CleanHistory,
    ShowToast(String),
    CloseToast(ToastId),
    Cosmic(cosmic::app::cosmic::Message),
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

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav.activate(id);
        self.nav
            .active_data()
            .map_or(Task::none(), |data: &Calculation| {
                self.calculation.expression = data.result.to_string().clone();
                self.calculation.result = String::new();
                self.calculation.display = data.expression.to_string();
                Task::none()
            })
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        for entry in &flags.config.history {
            nav.insert()
                .text(entry.to_string().clone())
                .data(entry.clone());
        }

        let about = About::default()
            .set_application_name(fl!("app-title"))
            .set_application_icon(Self::APP_ID)
            .set_developer_name("Eduardo Flores")
            .set_license_type("GPL-3.0")
            .set_version("0.1.0")
            .set_support_url("https://github.com/cosmic-utils/calculator/issues")
            .set_repository_url("https://github.com/cosmic-utils/calculator")
            .set_developers([("Eduardo Flores".into(), "edfloreshz@proton.me".into())]);

        let mut app = Calculator {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            nav,
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            calculation: Calculation::new(),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
        };

        let mut tasks = vec![];

        if let Some(id) = app.core.main_window_id() {
            tasks.push(app.set_window_title(fl!("app-title"), id));
        }

        (app, Task::batch(tasks))
    }

    fn about(&self) -> Option<&About> {
        Some(&self.about)
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
                            .push(wide_button(
                                Message::Operator(Operator::Clear),
                                Length::FillPortion(2),
                            ))
                            .push(standard_button(
                                Message::Operator(Operator::Modulus),
                                Length::FillPortion(1),
                            ))
                            .push(suggested_button(
                                Message::Operator(Operator::Divide),
                                Length::FillPortion(1),
                            ))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(
                                Message::Number(7.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Number(8.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Number(9.0),
                                Length::FillPortion(1),
                            ))
                            .push(suggested_button(
                                Message::Operator(Operator::Multiply),
                                Length::FillPortion(1),
                            ))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(
                                Message::Number(4.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Number(5.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Number(6.0),
                                Length::FillPortion(1),
                            ))
                            .push(suggested_button(
                                Message::Operator(Operator::Subtract),
                                Length::FillPortion(1),
                            ))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(
                                Message::Number(1.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Number(2.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Number(3.0),
                                Length::FillPortion(1),
                            ))
                            .push(suggested_button(
                                Message::Operator(Operator::Add),
                                Length::FillPortion(1),
                            ))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::with_capacity(4)
                            .push(standard_button(
                                Message::Number(0.0),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Operator(Operator::Point),
                                Length::FillPortion(1),
                            ))
                            .push(standard_button(
                                Message::Operator(Operator::Backspace),
                                Length::FillPortion(1),
                            ))
                            .push(suggested_button(
                                Message::Operator(Operator::Equal),
                                Length::FillPortion(1),
                            ))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .spacing(spacing.space_xs),
                    )
                    .push(
                        widget::row::row()
                            .push(widget::toaster(&self.toasts, widget::horizontal_space())),
                    )
                    .max_width(1000.0)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .spacing(spacing.space_xs),
            )
            .align_x(Alignment::Center)
            .spacing(spacing.space_s)
            .padding(spacing.space_xxs)
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
            Message::Cosmic(message) => {
                commands.push(cosmic::app::command::message(cosmic::app::message::cosmic(
                    message,
                )));
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

                self.set_context_title(context_page.title());
            }
            Message::Number(num) => self.calculation.on_number_press(num),
            Message::Input(input) => self.calculation.on_input(input),
            Message::Operator(operator) => match self.calculation.on_operator_press(&operator) {
                crate::calculation::Message::Continue => {
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
                crate::calculation::Message::Error(message) => {
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
            Message::NavMenuAction(action) => match action {
                NavMenuAction::Delete(entity) => {
                    if let Some(data) = self.nav.data::<Calculation>(entity).cloned() {
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
        Task::batch(commands)
    }

    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about_view()?.map(Message::Cosmic),
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

pub fn wide_button<'a>(message: Message, width: Length) -> Element<'a, Message> {
    let label = match message.clone() {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.display().to_string(),
        _ => String::new(),
    };
    button(label, message, theme::Button::Standard, width)
}

pub fn standard_button<'a>(message: Message, width: Length) -> Element<'a, Message> {
    let label = match message.clone() {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.display().to_string(),
        _ => String::new(),
    };
    button(label, message, theme::Button::Standard, width)
}

pub fn suggested_button<'a>(message: Message, width: Length) -> Element<'a, Message> {
    let label = match &message {
        Message::Number(num) => num.to_string(),
        Message::Operator(operator) => operator.display().to_string(),
        _ => String::new(),
    };
    button(label.to_string(), message, theme::Button::Suggested, width)
}

pub fn button<'a>(
    label: String,
    message: Message,
    style: theme::Button,
    width: Length,
) -> Element<'a, Message> {
    widget::button::custom(
        widget::container(widget::text(label).size(20.0))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .class(style)
    .width(width)
    .height(Length::Fill)
    .on_press(message)
    .into()
}
