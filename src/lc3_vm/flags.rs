#[allow(clippy::upper_case_acronyms)]
pub enum ConditionFlags {
    POS,
    ZRO,
    NEG,
}

impl From<ConditionFlags> for u16 {
    fn from(val: ConditionFlags) -> Self {
        match val {
            ConditionFlags::POS => 0,
            ConditionFlags::ZRO => 2,
            ConditionFlags::NEG => 4,
        }
    }
}
