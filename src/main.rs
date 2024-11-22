mod lc3_vm;
use lc3_vm::virtual_machine::VM;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vm = VM::default();
    vm.load_program("./2048.obj")?;
    Ok(vm.next_instruction()?)
}
