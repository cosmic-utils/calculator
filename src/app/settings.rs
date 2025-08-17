use std::sync::Mutex;

use crate::{
    app::{config::CalculatorConfig, Flags},
    core::{
        icons::{IconCache, ICON_CACHE},
        localization::localize,
    },
};
use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};

pub fn init() {
    localize();
    std::env::set_var("RUST_LOG", "cosmic_ext_calculator=info");
    pretty_env_logger::init();
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn settings() -> Settings {
    let config = CalculatorConfig::config();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(620.0));
    settings = settings.size(Size::new(360.0, 620.0));
    settings = settings.debug(false);
    settings
}

pub fn flags() -> Flags {
    let (config_handler, config) = (
        CalculatorConfig::config_handler(),
        CalculatorConfig::config(),
    );

    Flags {
        config_handler,
        config,
    }
}
