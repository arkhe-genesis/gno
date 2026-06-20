use anyhow::Result;
use cathedral_scheduler::TaskType;

pub struct Lc3Compiler;

impl Lc3Compiler {
    pub fn compile_task(_task: &str, task_type: TaskType) -> Result<String> {
        match task_type {
            TaskType::Inference => {
                Ok("
                    .ORIG x3000
                    LEA R0, PROMPT
                    PUTS
                    GETC
                    OUT
                    HALT
                    PROMPT .STRINGZ \"Enter something: \"
                ".to_string())
            }
            _ => {
                Ok("
                    .ORIG x3000
                    LEA R0, MSG
                    PUTS
                    HALT
                    MSG .STRINGZ \"Hello from LC-3 VM\"
                ".to_string())
            }
        }
    }

    pub fn assemble(asm: &str) -> Result<Vec<u16>> {
        crate::assembler::assemble(asm)
    }
}
