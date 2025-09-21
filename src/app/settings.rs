use std::sync::Mutex;

use crate::{
    app::{Flags, config::CalculatorConfig},
    core::{
        icons::{ICON_CACHE, IconCache},
        localization::localize,
    },
};
use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    localize();
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "cosmic_ext_calculator=info");
        }
    }
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn settings() -> Settings {
    let config = CalculatorConfig::config();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(270.0).min_height(450.0));
    settings = settings.size(Size::new(270.0, 450.0));
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
