// SPDX-License-Identifier: GPL-3.0-only

use app::CosmicCalculator;
mod app;
mod core;

use app::settings;

fn main() -> cosmic::iced::Result {
    settings::init();
    let (settings, flags) = (settings::settings(), settings::flags());
    cosmic::app::run::<CosmicCalculator>(settings, flags)
}
