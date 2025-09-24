// SPDX-License-Identifier: GPL-3.0-only

use std::any::TypeId;
use std::collections::HashMap;

use crate::app::{config::CONFIG_VERSION, operations::Calculator, operator::Operator};
use crate::core::{icons, key_binds::key_binds};
use crate::fl;
use cosmic::widget::RcElementWrapper;
use cosmic::{
    Application, ApplicationExt, Element,
    app::{Core, Task, context_drawer},
    cosmic_config,
    cosmic_config::Update,
    cosmic_theme,
    cosmic_theme::ThemeMode,
    iced::{
        Alignment, Event, Length, Subscription, event,
        keyboard::Event as KeyEvent,
        keyboard::{Key, Modifiers},
    },
    theme,
    widget::{
        self, ToastId,
        about::About,
        menu::{self, Action, ItemHeight, ItemWidth},
        nav_bar,
    },
};

mod config;
mod operations;
mod operator;
pub mod settings;

pub struct CosmicCalculator {
    core: Core,
    about: About,
    context_page: ContextPage,
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    nav: nav_bar::Model,
    modifiers: Modifiers,
    config_handler: Option<cosmic_config::Config>,
    config: config::CalculatorConfig,
    calculator: Calculator,
    toasts: widget::Toasts<Message>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Number(f32),
    Operator(Operator),
    Input(String),
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
    SetDecimalComma(bool),
    SetOutcome(Option<String>),
    Evaluate,
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

impl Application for CosmicCalculator {
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
            .map_or(Task::none(), |data: &Calculator| {
                self.calculator.expression = data.outcome.to_string();
                self.calculator.outcome = String::new();
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
            .developers([("Eduardo Flores", "edfloreshz@gmail.com")]);

        let mut app = CosmicCalculator {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: key_binds(),
            nav,
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            calculator: Calculator::new(),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
        };

        let mut tasks = vec![];

        tasks.push(app.set_window_title(fl!("app-title")));
        tasks.push(Task::perform(
            async move { operations::uses_decimal_comma().await },
            |decimal_comma| cosmic::Action::App(Message::SetDecimalComma(decimal_comma)),
        ));

        (app, Task::batch(tasks))
    }

    fn header_start<'a>(&'a self) -> Vec<Element<'a, Self::Message>> {
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
                widget::text_input("", &self.calculator.expression)
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
        let mut tasks = vec![];
        match message {
            Message::Open(url) => {
                if let Err(err) = open::that_detached(url) {
                    tracing::error!("{err}")
                }
            }
            Message::ShowToast(message) => {
                tasks.push(
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
            Message::SetDecimalComma(decimal_comma) => {
                self.calculator.decimal_comma = decimal_comma;
                tracing::info!("Calculator initialized");
            }
            Message::Number(num) => self.calculator.on_number_press(num),
            Message::Input(input) => self.calculator.on_input(input),
            Message::Operator(operator) => {
                if let Some(operations::Message::Evaluate) =
                    self.calculator.on_operator_press(&operator)
                {
                    tasks.push(self.update(Message::Evaluate));
                }
            }
            Message::Evaluate => {
                let expression = self.calculator.expression.trim().to_string();
                let calculator = self.calculator.clone();
                tasks.push(Task::perform(
                    async move {
                        operations::evaluate(&expression, calculator.decimal_comma).await
                    },
                    |outcome| cosmic::Action::App(Message::SetOutcome(outcome)),
                ));
            }
            Message::SetOutcome(outcome) => match outcome {
                Some(outcome) => {
                    let outcome = operations::extract_value(&outcome);
                    self.calculator.outcome = outcome.to_string();
                    let mut history = self.config.history.clone();
                    history.push(self.calculator.clone());
                    if let Some(config_handler) = &self.config_handler
                        && let Err(err) = self.config.set_history(config_handler, history)
                    {
                        tracing::error!("Failed to save history: {}", err);
                    }
                    self.nav
                        .insert()
                        .text(self.calculator.expression.clone())
                        .data(self.calculator.clone());
                    self.calculator.expression = outcome.to_string();
                }
                None => {
                    tracing::info!("No outcome");
                    let command = self.update(Message::ShowToast("No outcome".to_string()));
                    tasks.push(command);
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
                    if let Some(data) = self.nav.data::<Calculator>(entity).cloned() {
                        let mut history = self.config.history.clone();
                        history.retain(|calc| calc != &data);
                        if let Some(config_handler) = &self.config_handler
                            && let Err(err) = self.config.set_history(config_handler, history)
                        {
                            tracing::error!("Failed to save history: {}", err);
                        }
                        self.nav.remove(entity);
                    }
                }
            },
            Message::SystemThemeModeChange => {
                return self.update_config();
            }
            Message::CleanHistory => {
                if let Some(config_handler) = &self.config_handler
                    && let Err(err) = self.config.set_history(config_handler, vec![])
                {
                    tracing::error!("Failed to save history: {}", err);
                }
                self.nav.clear();
            }
        }
        Task::batch(tasks)
    }

    fn context_drawer<'a>(&'a self) -> Option<context_drawer::ContextDrawer<'a, Self::Message>> {
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

impl CosmicCalculator {
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
