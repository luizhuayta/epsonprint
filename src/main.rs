mod nucleo;
mod ui;

fn main() -> iced::Result {
    iced::application(ui::boot, ui::update, ui::view)
        .title("PrintDesk")
        .window_size(iced::Size::new(1080.0, 720.0))
        .subscription(ui::subscription)
        .run()
}
