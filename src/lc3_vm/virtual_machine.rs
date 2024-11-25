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
    #[error("Failure flags: {0}")]
    Flags(String),
    #[error("Failed to decode instruction: {0}")]
    Decode(String),
    #[error("Failed to update register: {0}")]
    GetRegister(String),
    #[error("Failed to read register: {0}")]
    ReadRegister(String),
    #[error("Failed to execute instruction: {0}")]
    Execute(String),
    #[error("Memory failure: {0}")]
    Memory(String),
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
    pub running: bool,
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
            running: false,
        }
    }
}

impl VM {
    pub fn load_program(&mut self, file_name: &str) -> Result<(), VMError> {
        let bytes = &std::fs::read(file_name)
            .map_err(|err| VMError::LoadProgram(format!("failed to read file: {}", err)))?;
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
        let pc = self.get_pc()?;
        let instruction = self
            .read_word(pc)
            .map_err(|err| VMError::Fetch(format!("failed to read: {}", err)))?
            .ok_or(VMError::Fetch(String::from("invalid Opcode")))?;
        let opcode = Self::decode(instruction).map_err(|err| VMError::Decode(err.to_string()))?;
        self.increment_pc();
        self.execute(opcode)?;

        Ok(())
    }

    fn read_word(&mut self, address: u16) -> Result<Option<u16>, VMError> {
        if let Some(word) = self.memory.get::<usize>(address.into()) {
            Ok(Some(*word))
        } else {
            Ok(None)
        }
    }

    fn store_word(&mut self, address: u16, value: u16) -> Result<(), VMError> {
        let memory = self
            .memory
            .get_mut::<usize>(address.into())
            .ok_or(VMError::Memory(String::from("invalid memory address")))?;
        *memory = value;
        Ok(())
    }

    fn decode(instruction: u16) -> Result<Opcode, OpcodeError> {
        Opcode::try_from(instruction)
    }

    fn execute(&mut self, opcode: Opcode) -> Result<(), VMError> {
        match opcode {
            Opcode::BR { n, z, p, offset } => {
                let flags_value = self
                    .get_flags()
                    .map_err(|err| VMError::Execute(format!("BR {}", err)))?;
                let offset = sign_extend_9_bits(offset);
                if n && flags_value == ConditionFlags::NEG.into() {
                    self.add_to_pc(offset);
                }
                if z && flags_value == ConditionFlags::ZRO.into() {
                    self.add_to_pc(offset);
                }
                if p && flags_value == ConditionFlags::POS.into() {
                    self.add_to_pc(offset);
                }
            }
            Opcode::ADD { dr, sr1, mode, sr2 } => {
                let source_register_1 = self
                    .get_register_value(sr1.into())
                    .map_err(|err| VMError::Execute(format!("ADD {}", err)))?;
                let rhs = if mode {
                    // imm mode
                    sign_extend_5_bits(sr2)
                } else {
                    self.get_register_value(sr2.into())
                        .map_err(|err| VMError::Execute(format!("ADD {}", err)))?
                };
                let result = source_register_1.wrapping_add(rhs);
                self.update_register(dr.into(), result)
                    .map_err(|err| VMError::Execute(format!("ADD {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("ADD {}", err)))?;
            }
            Opcode::LD { dr, offset } => {
                let pc_value = self
                    .get_pc()
                    .map_err(|err| VMError::Execute(format!("LD: {}", err)))?;
                let offset = sign_extend_9_bits(offset);
                // The address of the value is calculated by adding the incremented PC to the
                // sign extended offset
                let address = pc_value.wrapping_add(offset);
                // Read the word from the memory address
                let word = self
                    .read_word(address)
                    .map_err(|err| VMError::Execute(format!("LD: {}", err)))?
                    .ok_or(VMError::Execute(String::from("LD: read_word")))?;
                // Store the word into the destination register
                self.update_register(dr.into(), word)
                    .map_err(|err| VMError::Execute(format!("LD: {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("LD: {}", err)))?;
            }
            Opcode::ST { sr, offset } => {
                // Get the word to store from the source register
                let word = self
                    .get_register_value(sr.into())
                    .map_err(|err| VMError::Execute(format!("ST: {}", err)))?;

                let pc_value = self
                    .get_pc()
                    .map_err(|err| VMError::Execute(format!("ST: {}", err)))?;
                let offset = sign_extend_9_bits(offset);
                // The address of the value is calculated by adding the incremented PC to the
                // sign extended offset
                let address = pc_value.wrapping_add(offset);
                // Store the word into the calculated memory address
                self.store_word(address, word)
                    .map_err(|err| VMError::Execute(format!("ST: {}", err)))?;
            }
            Opcode::JSR { mode, offset } => {
                // Save PC into R7
                let pc_value = self
                    .get_pc()
                    .map_err(|err| VMError::Execute(format!("JSR: {}", err)))?;
                self.update_register(7, pc_value)
                    .map_err(|err| VMError::Execute(format!("JSR: {}", err)))?;
                // Calculate offset depending on mode
                let new_pc_value = if mode {
                    // If the mode flag is set the PC is the sum of the incremented PC and the sign
                    // extended offset
                    pc_value.wrapping_add(sign_extend_11_bits(offset))
                } else {
                    // If the mode flag is not set the pc is the base register, we shift the value 6 times
                    // to the right because the base address is stored in the 3 most significant bits of the offset
                    self.get_register_value(offset >> 6)
                        .map_err(|err| VMError::Execute(format!("JSR: {}", err)))?
                };
                // Jump PC
                self.set_pc(new_pc_value)
                    .map_err(|err| VMError::Execute(format!("JSR: {}", err)))?;
            }
            Opcode::AND { dr, sr1, mode, sr2 } => {
                let source_register_1 = self
                    .get_register_value(sr1.into())
                    .map_err(|err| VMError::Execute(format!("AND {}", err)))?;
                let rhs = if mode {
                    // imm mode
                    sign_extend_5_bits(sr2)
                } else {
                    self.get_register_value(sr2.into())
                        .map_err(|err| VMError::Execute(format!("AND {}", err)))?
                };
                // Bitwise AND
                let result = source_register_1 & rhs;
                // Save result into destination register
                self.update_register(dr.into(), result)
                    .map_err(|err| VMError::Execute(format!("AND {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("AND {}", err)))?;
            }
            Opcode::LDR { dr, base_r, offset } => {
                let base_register_value = self
                    .get_register(base_r.into())
                    .map_err(|err| VMError::Execute(format!("LDR: {}", err)))?;
                let offset = sign_extend_6_bits(offset);
                // Address is calculated by adding the base register value with sign extended offset
                let address = base_register_value.wrapping_add(offset);
                // Read word from calculated address
                let word = self
                    .read_word(address)
                    .map_err(|err| VMError::Execute(format!("LDR: {}", err)))?
                    .ok_or(VMError::Execute(String::from("LDR: read_word")))?;
                // Load read word into destination register
                self.update_register(dr.into(), word)
                    .map_err(|err| VMError::Execute(format!("LDR: {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("LDR: {}", err)))?;
            }
            Opcode::STR { sr, base_r, offset } => {
                let base_register_value = self
                    .get_register(base_r.into())
                    .map_err(|err| VMError::Execute(format!("STR: {}", err)))?;
                let offset = sign_extend_6_bits(offset);
                // Address is calculated by adding the base register value with sign extended offset
                let address = base_register_value.wrapping_add(offset);
                // Get word from regsiter
                let word = self
                    .get_register_value(sr.into())
                    .map_err(|err| VMError::Execute(format!("STR: {}", err)))?;
                // Store word into calculated address
                self.store_word(address, word)
                    .map_err(|err| VMError::Execute(format!("STR: {}", err)))?;
            }
            Opcode::RTI {} => {
                // This opcode is unused
                println!("unused")
            }
            Opcode::NOT { dr, sr } => {
                let source_register = self
                    .get_register_value(sr.into())
                    .map_err(|err| VMError::Execute(format!("NOT: {}", err)))?;
                // Bitwise NOT value
                let result = !source_register;
                // Save result into destination register
                self.update_register(dr.into(), result)
                    .map_err(|err| VMError::Execute(format!("NOT: {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("NOT: {}", err)))?;
            }
            Opcode::LDI { dr, offset } => {
                let pc_value = self
                    .get_pc()
                    .map_err(|err| VMError::Execute(format!("LDI: {}", err)))?;
                let offset = sign_extend_9_bits(offset);
                // The address of the address where the value we need to load is calculated
                // by adding the incremented PC to the sign extended offset
                let address_of_address = pc_value.wrapping_add(offset);
                // Using the previous address we read the final address where the target word is stored
                let address = self
                    .read_word(address_of_address)
                    .map_err(|err| VMError::Execute(format!("LDI: {}", err)))?
                    .ok_or(VMError::Execute(String::from(
                        "LDI: couldn't read first_address",
                    )))?;
                // Read the word from the final address
                let word = self
                    .read_word(address)
                    .map_err(|err| VMError::Execute(format!("LDI: {}", err)))?
                    .ok_or(VMError::Execute(String::from("LDI: read_word")))?;
                // Load read word into destintation address
                self.update_register(dr.into(), word)
                    .map_err(|err| VMError::Execute(format!("LDI: {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("LDI: {}", err)))?;
            }
            Opcode::STI { sr, offset } => {
                let pc_value = self
                    .get_pc()
                    .map_err(|err| VMError::Execute(format!("STI: {}", err)))?;
                let offset = sign_extend_9_bits(offset);
                // The address of the address where the value we need to store is calculated
                // by adding the incremented PC to the sign extended offset
                let address_of_address = pc_value.wrapping_add(offset);
                let address = self
                    .read_word(address_of_address)
                    .map_err(|err| VMError::Execute(format!("STI: {}", err)))?
                    .ok_or(VMError::Execute(String::from(
                        "STI: couldn't read first_address",
                    )))?;
                // Get the word from the register
                let word = self
                    .get_register_value(sr.into())
                    .map_err(|err| VMError::Execute(format!("STI: {}", err)))?;
                // Store the word into the calculated address
                self.store_word(address, word)
                    .map_err(|err| VMError::Execute(format!("STI: {}", err)))?;
            }
            Opcode::JMP { base_r } => {
                let offset = self
                    .get_register_value(base_r.into())
                    .map_err(|err| VMError::Execute(format!("JMP: {}", err)))?;
                // Unconditionaly set the PC to the value in the base register
                self.set_pc(offset)
                    .map_err(|err| VMError::Execute(format!("JMP: {}", err)))?;
            }
            Opcode::RES {} => {
                // This opcode is unused
                println!("unused");
            }
            Opcode::LEA { dr, offset } => {
                let pc_value = self
                    .get_pc()
                    .map_err(|err| VMError::Execute(format!("LEA: {}", err)))?;
                // The effective address is calculated by adding the incremented program counter
                // to the sign extended offset
                let address = pc_value.wrapping_add(sign_extend_9_bits(offset));
                // Load effective address into destination register
                self.update_register(dr.into(), address)
                    .map_err(|err| VMError::Execute(format!("LEA: {}", err)))?;

                self.update_flags(dr.into())
                    .map_err(|err| VMError::Execute(format!("LEA: {}", err)))?;
            }
            Opcode::TRAP { trap_vec } => {
                todo!("In next PR")
            }
        };
        Ok(())
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }

    fn update_flags(&mut self, register: u16) -> Result<bool, VMError> {
        let register_value = self
            .get_register(register)
            .map_err(|err| VMError::Flags(format!("read flags: {}", err)))?;
        let new_value = if (*register_value) == 0 {
            ConditionFlags::ZRO.into()
        } else if ((*register_value) >> 15) == 1 {
            ConditionFlags::NEG.into()
        } else {
            ConditionFlags::POS.into()
        };
        self.update_register(9, new_value)
            .map_err(|err| VMError::Flags(format!("update flags: {}", err)))?;
        Ok(true)
    }

    fn update_register(&mut self, register: u16, value: u16) -> Result<(), VMError> {
        let register_value = self.get_register(register)?;
        *register_value = value;
        Ok(())
    }

    fn get_register(&mut self, register: u16) -> Result<&mut u16, VMError> {
        let register_value: &mut u16 = match register {
            0 => &mut self.r0,
            1 => &mut self.r1,
            2 => &mut self.r2,
            3 => &mut self.r3,
            4 => &mut self.r4,
            5 => &mut self.r5,
            6 => &mut self.r6,
            7 => &mut self.r7,
            8 => &mut self.pc,
            9 => &mut self.cond,
            _ => return Err(VMError::GetRegister(format!("{register}"))),
        };
        Ok(register_value)
    }

    fn get_register_value(&self, register: u16) -> Result<u16, VMError> {
        let register_value: u16 = match register {
            0 => self.r0,
            1 => self.r1,
            2 => self.r2,
            3 => self.r3,
            4 => self.r4,
            5 => self.r5,
            6 => self.r6,
            7 => self.r7,
            8 => self.pc,
            9 => self.cond,
            _ => return Err(VMError::ReadRegister(format!("{register}"))),
        };
        Ok(register_value)
    }

    fn get_flags(&self) -> Result<u16, VMError> {
        self.get_register_value(9)
            .map_err(|err| VMError::Flags(format!("get flags: {}", err)))
    }

    fn add_to_pc(&mut self, offset: u16) {
        self.pc = self.pc.wrapping_add(offset);
    }

    fn get_pc(&self) -> Result<u16, VMError> {
        self.get_register_value(8)
            .map_err(|err| VMError::ProgramCounter(format!("get PC: {}", err)))
    }

    fn set_pc(&mut self, value: u16) -> Result<(), VMError> {
        self.update_register(8, value)
            .map_err(|err| VMError::ProgramCounter(format!("set PC: {}", err)))
    }
}

fn sign_extend_5_bits(num: u8) -> u16 {
    let mut num: u16 = num.into();
    if (num >> 4) == 1 {
        num |= 0b1111_1111_1110_0000;
    }
    num
}

fn sign_extend_6_bits(num: u8) -> u16 {
    let mut num: u16 = num.into();
    if (num >> 5) == 1 {
        num |= 0b1111_1111_1100_0000;
    }
    num
}

fn sign_extend_9_bits(mut num: u16) -> u16 {
    if (num >> 8) == 1 {
        num |= 0b1111_1110_0000_0000;
    }
    num
}

fn sign_extend_11_bits(mut num: u16) -> u16 {
    if (num >> 10) == 1 {
        num |= 0b1111_1000_0000_0000;
    }
    num
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sign_extend_5_bits_positive() {
        let num = sign_extend_5_bits(0b_0000_0001);
        assert_eq!(0b_0000_0000_0000_0001, num);
    }

    #[test]
    fn sign_extend_5_bits_negative() {
        let num = sign_extend_5_bits(0b_0000_0000_0001_1111);
        assert_eq!(0b_1111_1111_1111_1111, num);
    }

    #[test]
    fn add_with_overflow() -> Result<(), VMError> {
        let mut vm = VM::default();
        vm.load_program("./test-programs/add_overflow.obj")?;
        vm.next_instruction()?;
        assert_eq!(0b_1111_1111_1111_1111, vm.r0);
        vm.next_instruction()?;
        assert_eq!(0b_0000_0000_0000_0001, vm.r1);
        vm.next_instruction()?;
        assert_eq!(0b_0000_0000_0000_0000, vm.r1);
        Ok(())
    }

    #[test]
    fn for_loop() -> Result<(), VMError> {
        let mut vm = VM::default();
        vm.load_program("./test-programs/for_loop.obj")?;
        vm.next_instruction()?;
        for _ in 0..10 {
            vm.next_instruction()?;
            vm.next_instruction()?;
            vm.next_instruction()?;
        }
        assert_eq!(10, vm.r0);
        Ok(())
    }
}
