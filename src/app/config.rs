use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry},
    theme, Application,
};
use serde::{Deserialize, Serialize};

use crate::app::{calculation::Calculation, Calculator};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
pub struct CalculatorConfig {
    pub app_theme: AppTheme,
    pub history: Vec<Calculation>,
}

impl CalculatorConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(Calculator::APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> CalculatorConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                CalculatorConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    tracing::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => CalculatorConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}
