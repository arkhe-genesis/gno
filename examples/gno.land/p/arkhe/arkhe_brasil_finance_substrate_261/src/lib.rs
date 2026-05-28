//! Substrato 261 — ARKHE-BRASIL-FINANCE
//! Integração com SPB (STR), SPI (Pix), Clearing (B3/C3) e SPB-WEB.

pub mod types;
pub mod spb;
pub mod spi;
pub mod clearing;
pub mod web;
pub mod crypto;
pub mod bridge;

pub const SUBSTRATE_261_SEAL: &str = "f9fa264ea4bdff2ee104d986e3827eb823a2439909a50e658c1965ebbe787db1";
