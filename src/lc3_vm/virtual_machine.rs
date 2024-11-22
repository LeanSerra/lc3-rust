use thiserror::Error;

use super::{
    flags::ConditionFlags,
    opcodes::{Opcode, OpcodeError},
};
const MEMORY_MAX: usize = 1 << 16;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Failed to load program into memory: {0}")]
    LoadProgram(String),
    #[error("Failed to increment PC: {0}")]
    ProgramCounter(String),
    #[error("Failed to fetch instruction: {0}")]
    Fetch(String),
    #[error("Failed to update flags: {0}")]
    Flags(String),
    #[error("Failed to decode instruction: {0}")]
    Decode(String),
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

    pub fn next_instruction(&mut self) -> Result<(), VMError> {
        let instruction = self
            .fetch()
            .ok_or(VMError::Fetch(String::from("invalid Opcode")))?;
        let opcode = Self::decode(instruction).map_err(|err| VMError::Decode(err.to_string()))?;
        self.execute(opcode);
        self.increment_pc()?;

        Ok(())
    }

    fn fetch(&self) -> Option<u16> {
        let instruction: u16 = *self.memory.get::<usize>(self.pc.into())?;
        Some(instruction)
    }

    fn decode(instruction: u16) -> Result<Opcode, OpcodeError> {
        Opcode::try_from(instruction)
    }

    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::BR { n, z, p, offset } => {
                println!("{n}");
                println!("{z}");
                println!("{p}");
                println!("{offset}");
            }
            Opcode::ADD { dr, sr1, mode, sr2 } => {
                println!("{dr}");
                println!("{sr1}");
                println!("{mode}");
                println!("{sr2}");
            }
            Opcode::LD { dr, offset } => {
                println!("{dr}");
                println!("{offset}");
            }
            Opcode::ST { sr, offset } => {
                println!("{sr}");
                println!("{offset}");
            }
            Opcode::JSR { mode, offset } => {
                println!("{mode}");
                print!("{offset}");
            }
            Opcode::AND { dr, sr1, mode, sr2 } => {
                println!("{dr}");
                println!("{sr1}");
                println!("{mode}");
                println!("{sr2}");
            }
            Opcode::LDR { dr, base_r, offset } => {
                println!("{dr}");
                println!("{base_r}");
                println!("{offset}");
            }
            Opcode::STR { sr, base_r, offset } => {
                println!("{sr}");
                println!("{base_r}");
                println!("{offset}");
            }
            Opcode::RTI {} => {
                println!("unused")
            }
            Opcode::NOT { dr, sr } => {
                println!("{dr}");
                println!("{sr}");
            }
            Opcode::LDI { dr, offset } => {
                println!("{dr}");
                println!("{offset}");
            }
            Opcode::STI { sr, offset } => {
                println!("{sr}");
                println!("{offset}");
            }
            Opcode::JMP { base_r } => {
                println!("{base_r}");
            }
            Opcode::RES {} => {
                println!("unused");
            }
            Opcode::LEA { dr, offset } => {
                println!("{dr}");
                println!("{offset}");
            }
            Opcode::TRAP { trap_vec } => {
                println!("{trap_vec}");
            }
        }
    }

    fn increment_pc(&mut self) -> Result<(), VMError> {
        self.pc = self
            .pc
            .checked_add(1)
            .ok_or(VMError::ProgramCounter(String::from("Overflow")))?;
        Ok(())
    }

    fn update_flags(&mut self, register: u16) -> Result<bool, VMError> {
        let value: &mut u16 = match register {
            0 => &mut self.r0,
            1 => &mut self.r1,
            2 => &mut self.r2,
            3 => &mut self.r3,
            4 => &mut self.r4,
            5 => &mut self.r5,
            6 => &mut self.r6,
            7 => &mut self.r7,
            8 => &mut self.pc,
            9 => return Err(VMError::Flags(String::from("wrong register"))),
            10 => &mut self.count,
            _ => return Err(VMError::Flags(String::from("wrong register"))),
        };

        let value_copy = *value;

        *value = if *value == 0 {
            ConditionFlags::ZRO.into()
        } else if (*value >> 15) == 1 {
            ConditionFlags::NEG.into()
        } else {
            ConditionFlags::POS.into()
        };
        //check if shifting is done on a copy or if it modifies the value
        debug_assert_eq!(value_copy, *value);
        Ok(true)
    }
}
