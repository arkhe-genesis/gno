//! crates/cathedral-lc3-vm/src/vm.rs
//! LC-3 Virtual Machine executor
//! Selo: CATHEDRAL-ARKHE-LC3-VM-v3.0.1-2026-06-19

use std::collections::VecDeque;
use anyhow::{anyhow, Result};

// Constantes LC-3
const MEMORY_SIZE: usize = 1 << 16;
const REG_COUNT: usize = 10;
const PC_START: u16 = 0x3000;

// Registradores
const R_R0: usize = 0;
const R_R7: usize = 7;
const R_PC: usize = 8;
const R_COND: usize = 9;

// Flags de condição
const FL_POS: u16 = 1 << 0;
const FL_ZRO: u16 = 1 << 1;
const FL_NEG: u16 = 1 << 2;

// Opcodes
const OP_BR: u16 = 0;
const OP_ADD: u16 = 1;
const OP_LD: u16 = 2;
const OP_ST: u16 = 3;
const OP_JSR: u16 = 4;
const OP_AND: u16 = 5;
const OP_LDR: u16 = 6;
const OP_STR: u16 = 7;
const OP_RTI: u16 = 8;
const OP_NOT: u16 = 9;
const OP_LDI: u16 = 10;
const OP_STI: u16 = 11;
const OP_JMP: u16 = 12;
const OP_RES: u16 = 13;
const OP_LEA: u16 = 14;
const OP_TRAP: u16 = 15;

// Trap codes
const TRAP_GETC: u16 = 0x20;
const TRAP_OUT: u16 = 0x21;
const TRAP_PUTS: u16 = 0x22;
const TRAP_IN: u16 = 0x23;
const TRAP_PUTSP: u16 = 0x24;
const TRAP_HALT: u16 = 0x25;

/// Estrutura da VM LC-3
#[derive(Clone)]
pub struct Lc3Vm {
    memory: [u16; MEMORY_SIZE],
    reg: [u16; REG_COUNT],
    running: bool,
    // Buffer para entrada/saída (simulação)
    input_buffer: VecDeque<char>,
    output_buffer: String,
}

impl Lc3Vm {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            reg: [0; REG_COUNT],
            running: false,
            input_buffer: VecDeque::new(),
            output_buffer: String::new(),
        }
    }

    /// Carrega um programa binário na memória a partir do endereço de origem (PC_START)
    pub fn load_program(&mut self, program: &[u16]) {
        let start_addr = PC_START as usize;
        for (i, &word) in program.iter().enumerate() {
            if start_addr + i < MEMORY_SIZE {
                self.memory[start_addr + i] = word;
            }
        }
        self.reg[R_PC] = PC_START;
    }

    /// Define a entrada (simula teclado)
    pub fn set_input(&mut self, input: &str) {
        for ch in input.chars() {
            self.input_buffer.push_back(ch);
        }
    }

    /// Obtém a saída gerada pela VM
    pub fn get_output(&self) -> &str {
        &self.output_buffer
    }

    /// Limpa saída
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }

    /// Executa o programa carregado
    pub fn run(&mut self) -> Result<()> {
        self.running = true;
        while self.running {
            let pc = self.reg[R_PC] as usize;
            if pc >= MEMORY_SIZE {
                return Err(anyhow!("PC out of bounds"));
            }
            let instr = self.memory[pc];
            self.reg[R_PC] = self.reg[R_PC].wrapping_add(1);

            let op = (instr >> 12) & 0xF;
            match op {
                OP_BR => self.br(instr),
                OP_ADD => self.add(instr),
                OP_LD => self.ld(instr),
                OP_ST => self.st(instr),
                OP_JSR => self.jsr(instr),
                OP_AND => self.and(instr),
                OP_LDR => self.ldr(instr),
                OP_STR => self.str_(instr),
                OP_RTI => self.rti(),
                OP_NOT => self.not_(instr),
                OP_LDI => self.ldi(instr),
                OP_STI => self.sti(instr),
                OP_JMP => self.jmp(instr),
                OP_RES => { /* reserved, no-op */ }
                OP_LEA => self.lea(instr),
                OP_TRAP => self.trap(instr)?,
                _ => return Err(anyhow!("Invalid opcode")),
            }
        }
        Ok(())
    }

    fn br(&mut self, instr: u16) {
        let cond = (instr >> 9) & 0x7;
        let offset = instr & 0x1FF;
        let signed_offset = if (offset & 0x100) != 0 { offset | 0xFE00 } else { offset };
        if (cond & self.reg[R_COND]) != 0 {
            self.reg[R_PC] = self.reg[R_PC].wrapping_add_signed(signed_offset as i16);
        }
    }

    fn add(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let sr1 = ((instr >> 6) & 0x7) as usize;
        let imm_flag = (instr >> 5) & 0x1;
        if imm_flag != 0 {
            let imm5 = instr & 0x1F;
            let signed_imm = if (imm5 & 0x10) != 0 { imm5 | 0xFFE0 } else { imm5 };
            self.reg[dr] = self.reg[sr1].wrapping_add(signed_imm);
        } else {
            let sr2 = (instr & 0x7) as usize;
            self.reg[dr] = self.reg[sr1].wrapping_add(self.reg[sr2]);
        }
        self.update_cond(dr);
    }

    fn ld(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let pc_offset = instr & 0x1FF;
        let signed_offset = if (pc_offset & 0x100) != 0 { pc_offset | 0xFE00 } else { pc_offset };
        let addr = self.reg[R_PC].wrapping_add_signed(signed_offset as i16) as usize;
        self.reg[dr] = self.memory[addr];
        self.update_cond(dr);
    }

    fn st(&mut self, instr: u16) {
        let sr = ((instr >> 9) & 0x7) as usize;
        let pc_offset = instr & 0x1FF;
        let signed_offset = if (pc_offset & 0x100) != 0 { pc_offset | 0xFE00 } else { pc_offset };
        let addr = self.reg[R_PC].wrapping_add_signed(signed_offset as i16) as usize;
        self.memory[addr] = self.reg[sr];
    }

    fn jsr(&mut self, instr: u16) {
        self.reg[R_R7] = self.reg[R_PC];
        let flag = (instr >> 11) & 1;
        if flag == 0 {
            let base_r = ((instr >> 6) & 0x7) as usize;
            self.reg[R_PC] = self.reg[base_r];
        } else {
            let pc_offset = instr & 0x7FF;
            let signed_offset = if (pc_offset & 0x400) != 0 { pc_offset | 0xF800 } else { pc_offset };
            self.reg[R_PC] = self.reg[R_PC].wrapping_add_signed(signed_offset as i16);
        }
    }

    fn and(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let sr1 = ((instr >> 6) & 0x7) as usize;
        let imm_flag = (instr >> 5) & 0x1;
        if imm_flag != 0 {
            let imm5 = instr & 0x1F;
            let signed_imm = if (imm5 & 0x10) != 0 { imm5 | 0xFFE0 } else { imm5 };
            self.reg[dr] = self.reg[sr1] & signed_imm;
        } else {
            let sr2 = (instr & 0x7) as usize;
            self.reg[dr] = self.reg[sr1] & self.reg[sr2];
        }
        self.update_cond(dr);
    }

    fn ldr(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let base_r = ((instr >> 6) & 0x7) as usize;
        let offset = instr & 0x3F;
        let signed_offset = if (offset & 0x20) != 0 { offset | 0xFFC0 } else { offset };
        let addr = self.reg[base_r].wrapping_add_signed(signed_offset as i16) as usize;
        self.reg[dr] = self.memory[addr];
        self.update_cond(dr);
    }

    fn str_(&mut self, instr: u16) {
        let sr = ((instr >> 9) & 0x7) as usize;
        let base_r = ((instr >> 6) & 0x7) as usize;
        let offset = instr & 0x3F;
        let signed_offset = if (offset & 0x20) != 0 { offset | 0xFFC0 } else { offset };
        let addr = self.reg[base_r].wrapping_add_signed(signed_offset as i16) as usize;
        self.memory[addr] = self.reg[sr];
    }

    fn rti(&mut self) {
        // RTI is not fully implemented in this minimal VM without privilege rings
    }

    fn not_(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let sr = ((instr >> 6) & 0x7) as usize;
        self.reg[dr] = !self.reg[sr];
        self.update_cond(dr);
    }

    fn ldi(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let pc_offset = instr & 0x1FF;
        let signed_offset = if (pc_offset & 0x100) != 0 { pc_offset | 0xFE00 } else { pc_offset };
        let addr1 = self.reg[R_PC].wrapping_add_signed(signed_offset as i16) as usize;
        let addr2 = self.memory[addr1] as usize;
        self.reg[dr] = self.memory[addr2];
        self.update_cond(dr);
    }

    fn sti(&mut self, instr: u16) {
        let sr = ((instr >> 9) & 0x7) as usize;
        let pc_offset = instr & 0x1FF;
        let signed_offset = if (pc_offset & 0x100) != 0 { pc_offset | 0xFE00 } else { pc_offset };
        let addr1 = self.reg[R_PC].wrapping_add_signed(signed_offset as i16) as usize;
        let addr2 = self.memory[addr1] as usize;
        self.memory[addr2] = self.reg[sr];
    }

    fn jmp(&mut self, instr: u16) {
        let base_r = ((instr >> 6) & 0x7) as usize;
        self.reg[R_PC] = self.reg[base_r];
    }

    fn lea(&mut self, instr: u16) {
        let dr = ((instr >> 9) & 0x7) as usize;
        let pc_offset = instr & 0x1FF;
        let signed_offset = if (pc_offset & 0x100) != 0 { pc_offset | 0xFE00 } else { pc_offset };
        self.reg[dr] = self.reg[R_PC].wrapping_add_signed(signed_offset as i16);
        self.update_cond(dr);
    }

    fn trap(&mut self, instr: u16) -> Result<()> {
        let trap_code = instr & 0xFF;
        match trap_code {
            TRAP_GETC => {
                // Lê próximo caractere do buffer de entrada
                if let Some(ch) = self.input_buffer.pop_front() {
                    self.reg[R_R0] = ch as u16;
                } else {
                    self.reg[R_R0] = 0;
                }
            }
            TRAP_OUT => {
                let ch = self.reg[R_R0] as u8 as char;
                self.output_buffer.push(ch);
            }
            TRAP_PUTS => {
                let mut addr = self.reg[R_R0] as usize;
                while addr < MEMORY_SIZE {
                    let word = self.memory[addr];
                    if word == 0 { break; }
                    // Each word contains two chars (LC-3 is big-endian)
                    let c1 = (word >> 8) as u8 as char;
                    let c2 = (word & 0xFF) as u8 as char;
                    self.output_buffer.push(c1);
                    if c2 != '\0' {
                        self.output_buffer.push(c2);
                    }
                    addr += 1;
                }
            }
            TRAP_IN => {
                // Simples: usa primeiro caractere do buffer
                if let Some(ch) = self.input_buffer.pop_front() {
                    self.reg[R_R0] = ch as u16;
                    // Ecoa na saída
                    self.output_buffer.push(ch);
                } else {
                    self.reg[R_R0] = 0;
                }
            }
            TRAP_PUTSP => {
                // Similar a PUTS, mas armazenado em bytes alternados
                let mut addr = self.reg[R_R0] as usize;
                while addr < MEMORY_SIZE {
                    let word = self.memory[addr];
                    if word == 0 { break; }
                    let c1 = (word >> 8) as u8 as char;
                    let c2 = (word & 0xFF) as u8 as char;
                    self.output_buffer.push(c1);
                    if c2 != '\0' {
                        self.output_buffer.push(c2);
                    }
                    addr += 1;
                }
            }
            TRAP_HALT => {
                self.running = false;
            }
            _ => return Err(anyhow!("Unknown trap code: 0x{:X}", trap_code)),
        }
        Ok(())
    }

    fn update_cond(&mut self, reg_idx: usize) {
        let val = self.reg[reg_idx] as i16;
        self.reg[R_COND] = if val > 0 { FL_POS } else if val < 0 { FL_NEG } else { FL_ZRO };
    }
}
