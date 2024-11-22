use thiserror::Error;

use super::opcodes::Opcode;
const MEMORY_MAX: usize = 1 << 16;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Failed to load program into memory: {0}")]
    LoadProgram(String),
    #[error("Failed to increment PC: {0}")]
    ProgramCounter(String),
    #[error("Failed to fetch instruction {0}")]
    Fetch(String),
}

pub struct VM {
    memory: [u16; MEMORY_MAX],
    r0: u16,
    r1: u16,
    r2: u16,
    r3: u16,
    r4: u16,
    r5: u16,
    r6: u16,
    r7: u16,
    pc: u16,
    cond: u16,
    count: u16,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            memory: [0; MEMORY_MAX],
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            pc: 0x3000,
            cond: 0,
            count: 0,
        }
    }
}

impl VM {
    pub fn load_program(&mut self, file_name: &str) -> Result<(), VMError> {
        let bytes = &std::fs::read(file_name)
            .map_err(|_| VMError::LoadProgram(String::from("failed to read file")))?;
        self.load_bytes(bytes)?;
        Ok(())
    }

    fn load_bytes(&mut self, bytes: &[u8]) -> Result<(), VMError> {
        let mut loaded_memory = Vec::new();
        for two_bytes in bytes.chunks_exact(2) {
            let first_byte = two_bytes.first().ok_or(VMError::LoadProgram(String::from(
                "failed to read first byte from",
            )))?;
            // Cast into u16
            let first_byte: u16 = (*first_byte).into();
            let second_byte = two_bytes.get(1).ok_or(VMError::LoadProgram(String::from(
                "failed to read second byte from",
            )))?;
            // Cast into u16
            let second_byte: u16 = (*second_byte).into();

            // Join both bytes
            let mut joined_bytes = 0_u16;
            joined_bytes |= first_byte;
            joined_bytes <<= 8;
            joined_bytes |= second_byte;
            loaded_memory.push(joined_bytes);
        }
        self.memory
            .get_mut(..loaded_memory.len())
            .ok_or(VMError::LoadProgram(String::from(
                "Failed to write into VM memory",
            )))?
            .copy_from_slice(&loaded_memory);
        Ok(())
    }
}
