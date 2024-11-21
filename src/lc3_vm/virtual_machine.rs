use core::{error, fmt};

const MEMORY_MAX: usize = 1 << 16;

#[derive(Debug)]
pub enum VMError {
    LoadProgram(String),
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::LoadProgram(context) => {
                write!(f, "Failed to load program into memory: {context}")
            }
        }
    }
}

impl error::Error for VMError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            VMError::LoadProgram(_) => None,
        }
    }
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

#[allow(clippy::upper_case_acronyms)]
enum ConditionFlags {
    POS,
    ZRO,
    NEG,
    None,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            memory: [0; MEMORY_MAX],
            r0: 0_u16,
            r1: 0_u16,
            r2: 0_u16,
            r3: 0_u16,
            r4: 0_u16,
            r5: 0_u16,
            r6: 0_u16,
            r7: 0_u16,
            pc: 0x3000,
            cond: 0_u16,
            count: 0_u16,
        }
    }
}

impl VM {
    pub fn load_program(&mut self, file_name: &str) -> Result<(), VMError> {
        let bytes = &std::fs::read(file_name)
            .map_err(|_| VMError::LoadProgram(String::from("failed to read file")))?;
        let mut loaded_memory = Vec::new();
        for two_bytes in bytes.chunks_exact(2) {
            let first_byte = two_bytes.first().ok_or(VMError::LoadProgram(String::from(
                "failed to read first byte from",
            )))?;
            let second_byte = two_bytes.get(1).ok_or(VMError::LoadProgram(String::from(
                "failed to read second byte from",
            )))?;

            let mut concat_bytes = 0_u16;
            concat_bytes |= <u8 as std::convert::Into<u16>>::into(*first_byte);
            concat_bytes <<= 8;
            concat_bytes |= <u8 as std::convert::Into<u16>>::into(*second_byte);
            loaded_memory.push(concat_bytes);
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
