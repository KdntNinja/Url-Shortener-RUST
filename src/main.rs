use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "url-shortener")]
#[command(about = "A simple URL shortener CLI app", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Shorten {
        #[arg(short, long)]
        url: String,
    },
    Redirect {
        #[arg(short, long)]
        shortened_url: String,
    },
}

fn handle_url_shortening_request(url: &str) -> String {
    fn generate_shortened_url(url: &str) -> String {
        return url.to_string();
    }

    let new_url: String = generate_shortened_url(url);
    new_url
}

fn redirect_to_original_url(shortened_url: &str) -> String {
    shortened_url.to_string()
}

fn store_url_mapping(shortened_url: &str, original_url: &str) -> bool {
    true
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Shorten { url } => {
            let shortened_url = handle_url_shortening_request(url);
            println!("Shortened URL: {}", shortened_url);
        }
        Commands::Redirect { shortened_url } => {
            let original_url = redirect_to_original_url(shortened_url);
            println!("Original URL: {}", original_url);
        }
    }
}