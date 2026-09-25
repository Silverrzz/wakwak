use crate::engine::Engine;
use crate::util::Abort;
use std::ffi::{CStr, c_char};
use std::sync::{OnceLock, mpsc};

static COMMANDS: OnceLock<mpsc::Sender<String>> = OnceLock::new();

#[unsafe(no_mangle)]
extern "C" fn wakwak_ready() -> bool {
    COMMANDS.get().is_some()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn wakwak_command(input: *const c_char) -> i32 {
    if input.is_null() {
        return -1;
    }
    let Ok(input) = (unsafe { CStr::from_ptr(input) }).to_str() else {
        return -1;
    };
    let Some(sender) = COMMANDS.get() else {
        return 1;
    };
    match sender.send(input.to_owned()) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

pub fn run() {
    let mut engine = Engine::new();
    let (sender, receiver) = mpsc::channel();
    COMMANDS.set(sender).unwrap();

    for input in receiver {
        let input = input.trim();
        if !input.is_empty() && engine.handle_input(input) == Abort::Yes {
            break;
        }
    }
}
