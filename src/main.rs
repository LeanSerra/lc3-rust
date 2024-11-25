mod lc3_vm;
use lc3_vm::virtual_machine::VM;
use nix::{
    errno::Errno,
    sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg, Termios},
};
use std::{
    env,
    fs::File,
    os::fd::{AsFd, BorrowedFd},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("No filename provided")]
    NoFileName,
    #[error("Failed to read stdin {0}")]
    Stdin(String),
    #[error("Failed to get termios ERRNO: {0}")]
    GetTermios(String),
    #[error("Failed to disbale input buffering ERRNO: {0}")]
    DisableInputBuffering(String),
    #[error("Failed to restore input buffering ERRNO: {0}")]
    RestoreInputBuffering(String),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_name = env::args().nth(1).ok_or(MainError::NoFileName)?;
    let stdin_file = File::open("/dev/stdin").map_err(|err| MainError::Stdin(err.to_string()))?;
    let stdin_fd = AsFd::as_fd(&stdin_file);
    let mut termios =
        tcgetattr(stdin_fd).map_err(|err| MainError::DisableInputBuffering(err.to_string()))?;
    let original_termios = disable_input_buffering(stdin_fd, &mut termios)
        .map_err(|err| MainError::DisableInputBuffering(err.to_string()))?;

    let mut vm = VM::default();
    vm.load_program(&file_name)?;
    vm.running = true;
    while vm.running {
        vm.next_instruction()?;
    }

    restore_input_buffering(stdin_fd, original_termios)
        .map_err(|err| MainError::RestoreInputBuffering(err.to_string()))?;
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
