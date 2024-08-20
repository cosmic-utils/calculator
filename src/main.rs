// SPDX-License-Identifier: GPL-3.0-only

use app::Calculator;
mod app;
mod calculation;
mod config;
mod core;
mod operator;
mod settings;

fn main() -> cosmic::iced::Result {
    let (settings, flags) = settings::init();
    cosmic::app::run::<Calculator>(settings, flags)
}
