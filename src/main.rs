mod texture;
mod app;
mod camera;
mod entity;
mod light;
mod vertex;
fn main() {
    match app::run() {
        Ok(_) => {}
        Err(e) => eprintln!("{e}"),
    };
}
