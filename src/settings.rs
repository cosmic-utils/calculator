use crate::{app::Flags, config::CalculatorConfig, core::localization::set_localization};
use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};

pub fn init() -> (Settings, Flags) {
    set_localization();
    set_logger();
    let settings = get_app_settings();
    let flags = get_flags();
    (settings, flags)
}

pub fn get_app_settings() -> Settings {
    let config = CalculatorConfig::config();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(270.0).min_height(450.0));
    settings = settings.size(Size::new(270.0, 450.0));
    settings = settings.debug(false);
    settings
}

pub fn get_flags() -> Flags {
    let (config_handler, config) = (
        CalculatorConfig::config_handler(),
        CalculatorConfig::config(),
    );

    Flags {
        config_handler,
        config,
    }
}

pub fn set_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
}
