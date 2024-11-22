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
        let mut memory_chunks = bytes.chunks_exact(2);

        let origin = memory_chunks
            .next()
            .ok_or(VMError::LoadProgram(String::from(
                "failed to read origin address",
            )))?;
        let origin = Self::join_bytes(origin).ok_or(VMError::LoadProgram(String::from(
            "failed to read origin address",
        )))?;

        for two_bytes in memory_chunks {
            let joined_bytes = Self::join_bytes(two_bytes)
                .ok_or(VMError::LoadProgram(String::from("failed to read ")))?;
            loaded_memory.push(joined_bytes);
        }

        let loaded_byte_count: u16 = loaded_memory.len().try_into().map_err(|_| {
            VMError::LoadProgram(String::from("not enough memory to load the program"))
        })?;

        let last_memory_position =
            origin
                .checked_add(loaded_byte_count)
                .ok_or(VMError::LoadProgram(String::from(
                    "not enough memory to load the program",
                )))?;

        self.memory
            .get_mut(origin.into()..last_memory_position.into())
            .ok_or(VMError::LoadProgram(String::from(
                "failed to write into VM memory",
            )))?
            .copy_from_slice(&loaded_memory);
        Ok(())
    }

    fn join_bytes(bytes: &[u8]) -> Option<u16> {
        let first_byte = bytes.first()?;
        let first_byte: u16 = (*first_byte).into();
        let second_byte = bytes.get(1)?;
        let second_byte: u16 = (*second_byte).into();
        let mut joined_bytes = 0;
        joined_bytes |= first_byte;
        joined_bytes <<= 8;
        joined_bytes |= second_byte;
        Some(joined_bytes)
    }
}
