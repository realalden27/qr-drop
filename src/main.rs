use clap::Parser as _;
use qr_drop::config::Cli;
use qr_drop::server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let (payload, server_config) = match cli.resolve() {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        },
    };

    if let Err(e) = start_server(payload, server_config).await {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
