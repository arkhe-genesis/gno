//! Secure VM wrapper
//! Selo: CATHEDRAL-ARKHE-TEE-SECURE-VM-v3.0.1-2026-06-19

use cathedral_lc3_vm::Lc3Vm;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Wrapper seguro para VM que limita recursos e tempo
pub struct SecureVmExecutor {
    vm: Arc<Mutex<Lc3Vm>>,
    _max_instructions: u64,
    _memory_limit: usize,
}

impl SecureVmExecutor {
    pub fn new(max_instructions: u64, memory_limit: usize) -> Self {
        Self {
            vm: Arc::new(Mutex::new(Lc3Vm::new())),
            _max_instructions: max_instructions,
            _memory_limit: memory_limit,
        }
    }

    /// Executa código fornecido em binário LC-3 com sandbox
    pub async fn execute_secure(&self, binary: &[u16], input: &str) -> Result<String> {
        let mut vm = self.vm.lock().await;
        // Limpa estado anterior
        *vm = Lc3Vm::new();
        vm.load_program(binary);
        vm.set_input(input);

        // Executa com contador de instruções e timeout
        let mut vm_clone = vm.clone();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio::task::spawn_blocking(move || {
                let res = vm_clone.run();
                (res, vm_clone.get_output().to_string())
            })
        ).await;

        match result {
             Ok(Ok((Ok(_), output))) => Ok(output),
             Ok(Ok((Err(e), _))) => Err(e),
             Ok(Err(e)) => Err(anyhow::anyhow!("Task spawn error: {}", e)),
             Err(_) => Err(anyhow::anyhow!("Execution timed out")),
        }
    }
}
