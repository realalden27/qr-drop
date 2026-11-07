use clap::Parser as _;
use qr_drop::config::{Cli, CliAction};
use qr_drop::ensure_in_path;
use qr_drop::server::start_server;
use qr_drop::update::check_and_update;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ensure_in_path();

    let cli = Cli::parse();

    match cli.resolve() {
        Ok(CliAction::Serve { payload, config }) => {
            if let Err(e) = start_server(payload, config).await {
                eprintln!("Server error: {e}");
                std::process::exit(1);
            }
        },
        Ok(CliAction::Update) => {
            if let Err(e) = check_and_update().await {
                eprintln!("Update error: {e}");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        },
    }

    Ok(())
}
