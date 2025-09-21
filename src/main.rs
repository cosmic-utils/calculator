// SPDX-License-Identifier: GPL-3.0-only

use app::Calculator;
mod app;
mod core;
mod launcher;

use app::settings;

fn main() -> cosmic::iced::Result {
    settings::init();
    let (settings, flags) = (settings::settings(), settings::flags());
    cosmic::app::run::<Calculator>(settings, flags)
}
