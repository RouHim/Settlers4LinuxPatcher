mod display;
mod dynamic_patcher;
mod game_detection;
mod gui;
mod icons;
mod ini_handler;
mod patcher;
mod resolution;
mod theme;
mod validation;

use anyhow::Result;

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    if let Err(e) = run() {
        eprintln!("\n❌ Error: {}", e);

        // Print nested error causes for easier debugging
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("   Caused by: {}", err);
            source = err.source();
        }

        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    gui::run_gui().map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}
