#[allow(clippy::upper_case_acronyms)]
enum ConditionFlags {
    POS = 0,
    ZRO = 2,
    NEG = 4,
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
