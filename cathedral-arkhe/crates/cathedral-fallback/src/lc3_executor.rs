use cathedral_scheduler::TaskType;
use cathedral_lc3_vm::Lc3Vm;
use anyhow::Result;

pub struct Lc3VmExecutor {
    vm: Lc3Vm,
}

impl Lc3VmExecutor {
    pub fn new() -> Self {
        Self { vm: Lc3Vm::new() }
    }

    /// Carrega um programa binário LC-3
    pub fn load_binary(&mut self, program: &[u16]) {
        self.vm.load_program(program);
    }

    /// Define entrada da VM
    pub fn set_input(&mut self, input: &str) {
        self.vm.set_input(input);
    }
}

use crate::TaskExecutor;

#[async_trait::async_trait]
impl TaskExecutor for Lc3VmExecutor {
    fn execute(&self, task: &str, task_type: TaskType) -> Result<String> {
        let mut vm = self.vm.clone();

        let program = match task_type {
            TaskType::Inference => load_inference_program(),
            TaskType::MCTS => load_mcts_program(),
            _ => load_generic_program(),
        };
        vm.load_program(&program);
        vm.set_input(task);
        vm.run()?;
        Ok(vm.get_output().to_string())
    }

    async fn execute_async(&self, task: &str, task_type: TaskType) -> Result<String> {
        let task_clone = task.to_string();
        let this = self.clone();
        tokio::task::spawn_blocking(move || this.execute(&task_clone, task_type))
            .await
            .unwrap()
    }
}

impl Clone for Lc3VmExecutor {
    fn clone(&self) -> Self {
        Self { vm: self.vm.clone() }
    }
}


fn load_inference_program() -> Vec<u16> {
    vec![
        0x3000, // .ORIG x3000
        0xE0FF, // LEA R0, STR
        0xF022, // PUTS
        0xF025, // HALT
        // STR: "Hello World!"
        0x4865, 0x6C6C, 0x6F20, 0x576F, 0x726C, 0x6421, 0x0000,
    ]
}

fn load_mcts_program() -> Vec<u16> {
    vec![
        0x3000,
        0xF025,
    ]
}

fn load_generic_program() -> Vec<u16> {
     vec![
        0x3000,
        0xF025,
    ]
}
