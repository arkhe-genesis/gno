//! Cathedral ARKHE — xtask (versão expandida)
//! Selo: CATHEDRAL-ARKHE-XTASK-v2.0.0-2026-06-21

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::process::{Command, Stdio};
use std::time::Instant;
use which::which;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Cathedral ARKHE development tasks")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    CheckTools,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.command {
        Commands::CheckTools => check_tools()?,
    }

    println!("\n{}", "✅ Todas as verificações concluídas com sucesso!".green());
    println!("⏱️  Tempo total: {:.2}s", start.elapsed().as_secs_f64());
    Ok(())
}

fn check_tools() -> Result<()> {
    step("🔧 Verificando ferramentas instaladas");
    let tools = ["cargo"];
    let mut missing = Vec::new();
    for tool in &tools {
        if which(tool).is_ok() {
            println!("  ✅ {}", tool);
        } else {
            println!("  ❌ {} (não encontrado)", tool);
            missing.push(*tool);
        }
    }
    if !missing.is_empty() {
        return Err(anyhow!("Ferramentas faltando"));
    }
    Ok(())
}

fn step(msg: &str) {
    println!("\n{}", msg.bold().cyan());
    println!("{}", "─".repeat(60));
}
