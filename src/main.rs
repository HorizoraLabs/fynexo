mod app;
mod hud;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::run()
}