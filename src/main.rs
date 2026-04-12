mod app;
mod config;
mod dbus;
mod format;
mod icons;
mod model;
mod view;

fn main() -> cosmic::iced::Result {
    env_logger::init();
    cosmic::applet::run::<app::Vitals>(())
}
