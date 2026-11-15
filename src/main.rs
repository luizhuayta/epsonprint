mod nucleo;
mod ui;

fn main() -> iced::Result {
    iced::application(
        || {
            (
                ui::boot(),
                iced::system::theme().map(ui::Mensaje::TemaSistema),
            )
        },
        ui::update,
        ui::view,
    )
    .title("PrintDesk")
    .window_size(iced::Size::new(1080.0, 720.0))
    .window(iced::window::Settings {
        min_size: Some(iced::Size::new(760.0, 560.0)),
        exit_on_close_request: false,
        ..Default::default()
    })
    .font(include_bytes!("../assets/Inter-Regular.ttf").as_slice())
    .default_font(iced::Font::with_name("Inter"))
    .theme(ui::theme)
    .subscription(ui::subscription)
    .run()
}
