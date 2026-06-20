use anyhow::Result;

pub fn assemble(_asm: &str) -> Result<Vec<u16>> {
    // Mock assembler for LC-3
    Ok(vec![
        0x3000,
        0xE0FF,
        0xF022,
        0xF025,
    ])
}
