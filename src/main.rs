mod fs;
mod ui;
mod app;

fn main() {
    let mut app = app::App::new("/");
    ui::run(&mut app).unwrap();
}    