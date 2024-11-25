use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpcodeError {
    #[error("Invalid opcode")]
    InvalidOpcode,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Opcode {
    BR {
        n: bool,
        z: bool,
        p: bool,
        offset: u16,
    }, // branch
    ADD {
        dr: u8,
        sr1: u8,
        mode: bool,
        sr2: u8,
    }, // add
    LD {
        dr: u8,
        offset: u16,
    }, // load
    ST {
        sr: u8,
        offset: u16,
    }, // store
    JSR {
        mode: bool,
        offset: u16,
    }, // jump register
    AND {
        dr: u8,
        sr1: u8,
        mode: bool,
        sr2: u8,
    }, // bitwise and
    LDR {
        dr: u8,
        base_r: u8,
        offset: u8,
    }, // load register
    STR {
        sr: u8,
        base_r: u8,
        offset: u8,
    }, // store register
    RTI {}, // unused
    NOT {
        dr: u8,
        sr: u8,
    }, // bitwise not
    LDI {
        dr: u8,
        offset: u16,
    }, // load indirect
    STI {
        sr: u8,
        offset: u16,
    }, // store indirect
    JMP {
        base_r: u8,
    }, // jump
    RES {}, // reserved (unused)
    LEA {
        dr: u8,
        offset: u16,
    }, // load effective address
    TRAP {
        trap_vec: u8,
    }, // execute trap
}

impl TryFrom<u16> for Opcode {
    fn try_from(instruction: u16) -> Result<Self, Self::Error> {
        match instruction >> 12 {
            0 => {
                let n = ((instruction & 0b_0000_1000_0000_0000) >> 11) == 1;

                let z = ((instruction & 0b_0000_0100_0000_0000) >> 10) == 1;

                let p = ((instruction & 0b_0000_0010_0000_0000) >> 9) == 1;

                let offset = instruction & 0b_0000_0001_1111_1111;

                Ok(Opcode::BR { n, z, p, offset })
            }
            1 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let sr1 = (instruction & 0b0000_0001_1100_0000) >> 6;
                let mode = ((instruction & 0b0000_0000_0010_0000) >> 5) == 1;
                let sr2 = instruction & 0b0000_0000_0001_1111;

                Ok(Opcode::ADD {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    sr1: sr1.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    mode,
                    sr2: sr2.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            2 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let offset = instruction & 0b0000_0001_1111_1111;

                Ok(Opcode::LD {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset,
                })
            }
            3 => {
                let sr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let offset = instruction & 0b0000_0001_1111_1111;
                Ok(Opcode::ST {
                    sr: sr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset,
                })
            }
            4 => {
                let mode = ((instruction & 0b0000_1000_0000_0000) >> 11) == 1;
                let offset = instruction & 0b0000_0111_1111_1111;
                Ok(Opcode::JSR { mode, offset })
            }
            5 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let sr1 = (instruction & 0b0000_0001_1100_0000) >> 6;
                let mode = ((instruction & 0b0000_0000_0010_0000) >> 5) == 1;
                let sr2 = instruction & 0b0000_0000_0001_1111;

                Ok(Opcode::AND {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    sr1: sr1.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    mode,
                    sr2: sr2.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            6 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let base_r = (instruction & 0b0000_0001_1100_0000) >> 6;
                let offset = instruction & 0b0000_0000_0011_1111;

                Ok(Opcode::LDR {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    base_r: base_r.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset: offset.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            7 => {
                let sr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let base_r = (instruction & 0b0000_0001_1100_0000) >> 6;
                let offset = instruction & 0b0000_0000_0011_1111;

                Ok(Opcode::STR {
                    sr: sr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    base_r: base_r.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset: offset.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            8 => Ok(Opcode::RTI {}),
            9 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let sr = (instruction & 0b0000_0001_1100_0000) >> 6;

                Ok(Opcode::NOT {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    sr: sr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            10 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let offset = instruction & 0b0000_0001_1111_1111;
                Ok(Opcode::LDI {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset,
                })
            }
            11 => {
                let sr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let offset = instruction & 0b0000_0001_1111_1111;
                Ok(Opcode::STI {
                    sr: sr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset,
                })
            }
            12 => {
                let base_r = (instruction & 0b0000_0001_1100_0000) >> 6;

                Ok(Opcode::JMP {
                    base_r: base_r.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            13 => Ok(Opcode::RES {}),
            14 => {
                let dr = (instruction & 0b0000_1110_0000_0000) >> 9;
                let offset = instruction & 0b0000_0001_1111_1111;

                Ok(Opcode::LEA {
                    dr: dr.try_into().map_err(|_| OpcodeError::InvalidOpcode)?,
                    offset,
                })
            }
            15 => {
                let trap_vec = instruction & 0b0000_0000_1111_1111;

                Ok(Opcode::TRAP {
                    trap_vec: trap_vec
                        .try_into()
                        .map_err(|_| OpcodeError::InvalidOpcode)?,
                })
            }
            _ => Err(OpcodeError::InvalidOpcode),
        }
    }
    type Error = OpcodeError;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_br() -> Result<(), OpcodeError> {
        let br = Opcode::BR {
            n: false,
            z: false,
            p: true,
            offset: 4,
        };
        let instruction: u16 = 0b_0000_0010_0000_0100;
        assert_eq!(br, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_add_mode_0() -> Result<(), OpcodeError> {
        let add = Opcode::ADD {
            dr: 1,
            sr1: 2,
            mode: false,
            sr2: 3,
        };
        let instruction: u16 = 0b_0001_0010_1000_0011;
        assert_eq!(add, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_add_mode_1() -> Result<(), OpcodeError> {
        let add = Opcode::ADD {
            dr: 1,
            sr1: 2,
            mode: true,
            sr2: 11, //imm5
        };
        let instruction: u16 = 0b_0001_0010_1010_1011;
        assert_eq!(add, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_ld() -> Result<(), OpcodeError> {
        let ld = Opcode::LD { dr: 4, offset: 511 };
        let instruction = 0b_0010_1001_1111_1111;
        assert_eq!(ld, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_st() -> Result<(), OpcodeError> {
        let st = Opcode::ST { sr: 4, offset: 255 };
        let instruction = 0b_0011_1000_1111_1111;
        assert_eq!(st, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_jsr_mode_1() -> Result<(), OpcodeError> {
        let jsr = Opcode::JSR {
            mode: true,
            offset: 2047,
        };
        let instruction = 0b_0100_1111_1111_1111;
        assert_eq!(jsr, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_and_mode_0() -> Result<(), OpcodeError> {
        let and = Opcode::AND {
            dr: 1,
            sr1: 2,
            mode: false,
            sr2: 3,
        };
        let instruction = 0b_0101_0010_1000_0011;
        assert_eq!(and, Opcode::try_from(instruction)?);
        Ok(())
    }

    #[test]
    fn parse_and_mode_1() -> Result<(), OpcodeError> {
        let and = Opcode::AND {
            dr: 1,
            sr1: 2,
            mode: true,
            sr2: 11, //imm5
        };
        let instruction: u16 = 0b_0101_0010_1010_1011;
        assert_eq!(and, Opcode::try_from(instruction)?);
        Ok(())
    }
}
