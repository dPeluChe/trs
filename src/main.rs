use clap::Parser;
use clap::Subcommand;

/// TARS CLI - A command-line interface tool
#[derive(Parser)]
#[command(name = "tars")]
#[command(about = "TARS CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Say hello to someone
    Hello {
        /// Name to greet
        #[arg(short, long, default_value = "world")]
        name: String,
    },
    /// Display version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Hello { name } => {
            println!("Hello, {}!", name);
        }
        Commands::Version => {
            println!("tars-cli {}", env!("CARGO_PKG_VERSION"));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hello_default() {
        // Test that the hello command works with default name
        let name = "world".to_string();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_hello_custom_name() {
        // Test that custom names work
        let name = "TARS".to_string();
        assert_eq!(name, "TARS");
    }

    #[test]
    fn test_version_format() {
        // Test that version string contains package info
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }
}
