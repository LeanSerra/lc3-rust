#[allow(clippy::upper_case_acronyms)]
enum Opcodes {
    BR,   // branch
    ADD,  // add
    LD,   // load
    ST,   // store
    JSR,  // jump register
    AND,  // bitwise and
    LDR,  // load register
    STR,  // store register
    RTI,  // unused
    NOT,  // bitwise not
    LDI,  // load indirect
    STI,  // store indirect
    JMP,  // jump
    RES,  // reserved (unused)
    LEA,  // load effective address
    TRAP, // execute trap
}
