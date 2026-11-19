mod app;
mod calcrpn;
mod commands;
mod finance;
mod highlight;
mod input;
mod ui;

fn main() -> std::io::Result<()> {
    // ratatui::init はパニック時に端末を元に戻すフックも設定する
    let mut terminal = ratatui::init();
    let result = app::App::new().run(&mut terminal);
    ratatui::restore();
    result
}
