use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cathedral-cli")]
#[command(about = "Cathedral LLM HTTP CLI", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate text via HTTP API
    Generate {
        #[arg(short, long)]
        prompt: String,

        #[arg(short, long, default_value = "did:cathedral:agent:default")]
        did: String,

        #[arg(short, long, default_value = "L1")]
        level: String,

        #[arg(long, default_value = "false")]
        show_thinking: bool,
    },

    /// Query memory
    Memory {
        #[arg(short, long)]
        did: String,
    },

    /// Check status
    Status {
        #[arg(short, long)]
        did: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base_url = "http://localhost:8080";

    match cli.command {
        Commands::Generate { prompt, did, level, show_thinking } => {
            println!("Generating response for: {}", prompt);

            let req_body = serde_json::json!({
                "prompt": prompt,
                "did": did,
                "signature": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                "level": level
            });

            match client.post(format!("{}/generate", base_url)).json(&req_body).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        let json: serde_json::Value = res.json().await.unwrap();
                        println!("Response: {}", json["text"].as_str().unwrap_or(""));
                        if show_thinking {
                            if let Some(thinking) = json.get("thinking").and_then(|t| t.as_str()) {
                                println!("Thinking: {}", thinking);
                            }
                        }
                    } else {
                        eprintln!("API returned error: {}", res.status());
                    }
                }
                Err(e) => eprintln!("Failed to connect to API: {}", e),
            }
        }
        Commands::Memory { did } => {
            println!("Querying memory for DID: {}", did);
            match client.get(format!("{}/memory/{}", base_url, did)).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        let json: serde_json::Value = res.json().await.unwrap();
                        if let Some(arr) = json.as_array() {
                            for m in arr {
                                println!("- {}", m["content"].as_str().unwrap_or(""));
                            }
                        }
                    } else {
                        eprintln!("API returned error: {}", res.status());
                    }
                }
                Err(e) => eprintln!("Failed to connect to API: {}", e),
            }
        }
        Commands::Status { did } => {
            println!("Querying status for DID: {}", did);
            match client.get(format!("{}/status/{}", base_url, did)).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        let json: serde_json::Value = res.json().await.unwrap();
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    } else {
                        eprintln!("API returned error: {}", res.status());
                    }
                }
                Err(e) => eprintln!("Failed to connect to API: {}", e),
            }
        }
    }
}
