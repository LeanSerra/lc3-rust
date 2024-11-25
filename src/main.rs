mod lc3_vm;
use lc3_vm::virtual_machine::VM;
use nix::{
    errno::Errno,
    sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg, Termios},
};
use std::{
    fs::File,
    os::fd::{AsFd, BorrowedFd},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin_file = File::open("/dev/stdin")?;
    let stdin_fd = AsFd::as_fd(&stdin_file);
    let mut termios = tcgetattr(stdin_fd)?;
    let original_termios = disable_input_buffering(stdin_fd, &mut termios)?;

    let mut vm = VM::default();
    vm.load_program("./2048.obj")?;
    vm.running = true;
    while vm.running {
        vm.next_instruction()?;
    }

    restore_input_buffering(stdin_fd, original_termios)?;
    Ok(())
}

fn disable_input_buffering(stdin_fd: BorrowedFd, termios: &mut Termios) -> Result<Termios, Errno> {
    let original_termios = termios.clone();
    let mut flags = termios.local_flags;
    let flag_echo = LocalFlags::ECHO;
    let flag_icanon = LocalFlags::ICANON;
    flags.toggle(flag_echo);
    flags.toggle(flag_icanon);
    termios.local_flags = flags;
    tcsetattr(stdin_fd, SetArg::TCSANOW, termios)?;
    Ok(original_termios)
}

fn restore_input_buffering(stdin_fd: BorrowedFd, original_termios: Termios) -> Result<(), Errno> {
    tcsetattr(stdin_fd, SetArg::TCSANOW, &original_termios)?;
    Ok(())
}
