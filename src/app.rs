// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::collections::HashMap;

use crate::app::calculation::Calculation;
use crate::app::{config::CONFIG_VERSION, operator::Operator};
use crate::core::{icons, key_binds::key_binds};
use crate::{fl, launcher};
use cosmic::app::context_drawer;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::Update;
use cosmic::cosmic_theme::ThemeMode;
use cosmic::iced::{
    Alignment, Event, Length, Subscription, event,
    keyboard::Event as KeyEvent,
    keyboard::{Key, Modifiers},
};
use cosmic::widget::about::About;
use cosmic::widget::menu::{Action, ItemHeight, ItemWidth};
use cosmic::widget::{self, RcElementWrapper, ToastId, menu, nav_bar};
use cosmic::{Application, ApplicationExt, Element, cosmic_config, cosmic_theme, theme};
use tokio::sync::mpsc;

mod calculation;
mod config;
mod operator;
pub mod settings;

pub struct Calculator {
    core: Core,
    about: About,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    nav: nav_bar::Model,
    modifiers: Modifiers,
    config_handler: Option<cosmic_config::Config>,
    config: config::CalculatorConfig,
    tx: Option<mpsc::Sender<launcher::Request>>,
    calculation: Calculation,
    toasts: widget::Toasts<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Number(i32),
    Operator(Operator),
    Input(String),
    LauncherEvent(launcher::Event),
    ToggleContextPage(ContextPage),
    ToggleContextDrawer,
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    SystemThemeModeChange,
    NavMenuAction(NavMenuAction),
    CleanHistory,
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
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::NavMenuAction(*self))
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
                Task::none()
            })
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        for entry in &flags.config.history {
            nav.insert()
                .text(entry.expression.clone())
                .data(entry.clone());
        }

        let about = About::default()
            .name(fl!("app-title"))
            .icon(widget::icon::from_name(Self::APP_ID))
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
            nav,
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            tx: None,
            calculation: Calculation::new(),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
        };

        let mut tasks = vec![];

        tasks.push(app.set_window_title(fl!("app-title")));

        (app, Task::batch(tasks))
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            RcElementWrapper::new(menu::root(fl!("view")).into()),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(
                        fl!("clear-history"),
                        Some(icons::get_handle("large-brush-symbolic", 14)),
                        MenuAction::ClearHistory,
                    ),
                    menu::Item::Button(
                        fl!("about"),
                        Some(icons::get_handle("settings-symbolic", 14)),
                        MenuAction::About,
                    ),
                ],
            ),
        )])
        .item_height(ItemHeight::Dynamic(40))
        .item_width(ItemWidth::Uniform(240))
        .spacing(4.0);

        vec![menu_bar.into()]
    }

    fn nav_context_menu(
        &self,
        id: nav_bar::Id,
    ) -> Option<Vec<menu::Tree<cosmic::Action<Self::Message>>>> {
        Some(cosmic::widget::menu::items(
            &HashMap::new(),
            vec![cosmic::widget::menu::Item::Button(
                fl!("delete"),
                Some(icons::get_handle("user-trash-symbolic", 14)),
                NavMenuAction::Delete(id),
            )],
        ))
    }

    fn view<'a>(&'a self) -> Element<'a, Self::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;

        widget::column::with_capacity(2)
            .push(
                widget::text_input("", &self.calculation.expression)
                    .on_input(Message::Input)
                    .on_submit(|_| Message::Operator(Operator::Equal))
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
                            .push(standard_button(Message::Number(7), Length::FillPortion(1)))
                            .push(standard_button(Message::Number(8), Length::FillPortion(1)))
                            .push(standard_button(Message::Number(9), Length::FillPortion(1)))
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
                            .push(standard_button(Message::Number(4), Length::FillPortion(1)))
                            .push(standard_button(Message::Number(5), Length::FillPortion(1)))
                            .push(standard_button(Message::Number(6), Length::FillPortion(1)))
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
                            .push(standard_button(Message::Number(1), Length::FillPortion(1)))
                            .push(standard_button(Message::Number(2), Length::FillPortion(1)))
                            .push(standard_button(Message::Number(3), Length::FillPortion(1)))
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
                            .push(standard_button(Message::Number(0), Length::FillPortion(1)))
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
                                tracing::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        tracing::warn!(
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
                    tracing::error!("{err}")
                }
            }
            Message::LauncherEvent(event) => match event {
                launcher::Event::Started(tx) => {
                    self.tx.replace(tx);
                }
                launcher::Event::ServiceIsClosed => {
                    self.request(launcher::Request::ServiceIsClosed);
                }
                launcher::Event::Response(response) => match response {
                    pop_launcher::Response::Close => {
                        tracing::info!("Closed launcher");
                        self.calculation.clear();
                    }
                    pop_launcher::Response::Fill(s) => {
                        tracing::info!("Responded with {s}");
                        self.calculation.expression = s;
                        self.request(launcher::Request::Search(
                            self.calculation.expression.clone(),
                        ));
                    }
                    pop_launcher::Response::Update(results) => {
                        if results.is_empty() {
                            self.calculation.expression = "".to_string();
                        }

                        match results.get(0) {
                            Some(result) => {
                                tracing::info!("Result is: {}", result.name);
                                if let Ok(_) = result.name.parse::<f64>() {
                                    let mut history = self.config.history.clone();
                                    self.calculation.result = result.name.clone();
                                    history.push(self.calculation.clone());
                                    config_set!(history, history);
                                    self.nav
                                        .insert()
                                        .text(self.calculation.expression.clone())
                                        .data(self.calculation.clone());

                                    self.calculation.expression = result.name.clone();
                                } else if result.name.contains('≈') {
                                    let mut history = self.config.history.clone();
                                    self.calculation.result = result.name.clone();
                                    history.push(self.calculation.clone());
                                    config_set!(history, history);
                                    self.nav
                                        .insert()
                                        .text(self.calculation.expression.clone())
                                        .data(self.calculation.clone());

                                    self.calculation.expression = result.name.clone();
                                }
                            }
                            None => {
                                let command = self.update(Message::ShowToast(
                                    "Unable to compute result".to_string(),
                                ));
                                commands.push(command);
                            }
                        }
                    }
                    _ => {
                        tracing::info!("Other option reached");
                    }
                },
            },
            Message::ShowToast(message) => {
                commands.push(
                    self.toasts
                        .push(widget::toaster::Toast::new(message))
                        .map(cosmic::Action::App),
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
                Some(crate::app::calculation::Message::Calculate(mut expression)) => {
                    if expression.contains('≈') {
                        expression = expression.replace("≈", "").trim().to_string();
                    }
                    self.request(launcher::Request::Search(expression));
                }
                None => {
                    tracing::info!("Operator: {:?}", operator);
                    if operator == Operator::Equal {
                        let mut history = self.config.history.clone();
                        history.push(self.calculation.clone());
                        config_set!(history, history);
                        self.nav
                            .insert()
                            .text(self.calculation.expression.clone())
                            .data(self.calculation.clone());
                        self.calculation.expression = self.calculation.result.to_string();
                    }
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
            launcher::subscription(0).map(Message::LauncherEvent),
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
                    tracing::info!(
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
                    tracing::info!(
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
    fn request(&self, r: launcher::Request) {
        tracing::debug!("request: {:?}", r);
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.blocking_send(r) {
                tracing::error!("tx: {e}");
            }
        } else {
            tracing::info!("tx not found");
        }
    }

    fn update_config(&mut self) -> Task<Message> {
        cosmic::command::set_theme(self.config.app_theme.theme())
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
