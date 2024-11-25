use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrapError {
    #[error("Invalid trap code {0}")]
    InvalidTrap(u8),
}

pub enum Trap {
    GetC,
    Out,
    Puts,
    In,
    Putsp,
    Halt,
}

impl TryFrom<u8> for Trap {
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Trap::GetC),
            0x21 => Ok(Trap::Out),
            0x22 => Ok(Trap::Puts),
            0x23 => Ok(Trap::In),
            0x24 => Ok(Trap::Putsp),
            0x25 => Ok(Trap::Halt),
            _ => Err(TrapError::InvalidTrap(value)),
        }
    }

    type Error = TrapError;
}
