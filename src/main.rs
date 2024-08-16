// SPDX-License-Identifier: GPL-3.0-only

use app::Calculator;
use cosmic::iced::{Limits, Size};
mod app;
mod core;

fn main() -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default().size_limits(Limits::new(
        Size::new(270.0, 450.0),
        Size::new(270.0, 450.0),
    ));
    cosmic::app::run::<Calculator>(settings, ())
}
