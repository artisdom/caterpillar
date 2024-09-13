use std::sync::Mutex;

use capi_ffi::{framed_buffer::FramedBuffer, shared::Shared};
use capi_protocol::{
    CODE_BUFFER_SIZE, COMMANDS_BUFFER_SIZE, UPDATES_BUFFER_SIZE,
};
use tokio::sync::mpsc::error::TryRecvError;

use crate::debugger::Debugger;

pub static STATE: Mutex<Option<Debugger>> = Mutex::new(None);

static UPDATES: Shared<FramedBuffer<UPDATES_BUFFER_SIZE>> =
    Shared::new(FramedBuffer::new());
static CODE: Shared<FramedBuffer<CODE_BUFFER_SIZE>> =
    Shared::new(FramedBuffer::new());
static COMMANDS: Shared<FramedBuffer<COMMANDS_BUFFER_SIZE>> =
    Shared::new(FramedBuffer::new());

/// See comment on `capi_runtime::ffi::LAST_UPDATE_READ`
static LAST_UPDATE_WRITE: Mutex<Option<(usize, usize)>> = Mutex::new(None);

#[no_mangle]
pub fn updates_write(len: usize) {
    // Sound, because the reference is dropped before we give back control to
    // the host.
    let buffer = unsafe { UPDATES.access() };
    let update = buffer.write_frame(len);

    *LAST_UPDATE_WRITE.lock().unwrap() =
        Some((update.as_ptr() as usize, update.len()));
}

#[no_mangle]
pub fn updates_write_ptr() -> usize {
    let (ptr, _) = LAST_UPDATE_WRITE.lock().unwrap().unwrap();
    ptr
}

#[no_mangle]
pub fn updates_write_len() -> usize {
    let (_, len) = LAST_UPDATE_WRITE.lock().unwrap().unwrap();
    len
}

static LAST_CODE_READ: Mutex<Option<(usize, usize)>> = Mutex::new(None);

#[no_mangle]
pub fn code_read() {
    let mut state = STATE.lock().unwrap();
    let state = state.get_or_insert_with(Default::default);

    if state.code_rx.has_changed().ok().unwrap_or(false) {
        let code = state.code_rx.borrow_and_update();
        let code = ron::to_string(&*code).unwrap();

        // Sound, because the reference is dropped before we call the method
        // again or we give back control to the host.
        let buffer = unsafe { CODE.access() };
        buffer
            .write_frame(code.len())
            .copy_from_slice(code.as_bytes());
    }

    // Sound, because the reference is dropped before we give back control to
    // the host.
    let buffer = unsafe { CODE.access() };
    let code = buffer.read_frame();

    *LAST_CODE_READ.lock().unwrap() =
        Some((code.as_ptr() as usize, code.len()));
}

#[no_mangle]
pub fn code_read_ptr() -> usize {
    let (ptr, _) = LAST_CODE_READ.lock().unwrap().unwrap();
    ptr
}

#[no_mangle]
pub fn code_read_len() -> usize {
    let (_, len) = LAST_CODE_READ.lock().unwrap().unwrap();
    len
}

static LAST_COMMAND_READ: Mutex<Option<(usize, usize)>> = Mutex::new(None);

#[no_mangle]
pub fn commands_read() {
    let mut state = STATE.lock().unwrap();
    let state = state.get_or_insert_with(Default::default);

    loop {
        let command = match state.commands_to_runtime_rx.try_recv() {
            Ok(command) => command,
            Err(TryRecvError::Empty) => {
                break;
            }
            Err(TryRecvError::Disconnected) => {
                // The other end has hung up, which happens during
                // shutdown. Shut down this task, too.
                return;
            }
        };

        // Sound, because the reference is dropped before we call the method
        // again or we give back control to the host.
        let buffer = unsafe { COMMANDS.access() };
        buffer.write_frame(command.len()).copy_from_slice(&command);
    }

    // Sound, because the reference is dropped before we give back control to
    // the host.
    let buffer = unsafe { COMMANDS.access() };
    let command = buffer.read_frame();

    *LAST_COMMAND_READ.lock().unwrap() =
        Some((command.as_ptr() as usize, command.len()));
}

#[no_mangle]
pub fn commands_read_ptr() -> usize {
    let (ptr, _) = LAST_COMMAND_READ.lock().unwrap().unwrap();
    ptr
}

#[no_mangle]
pub fn commands_read_len() -> usize {
    let (_, len) = LAST_COMMAND_READ.lock().unwrap().unwrap();
    len
}

#[no_mangle]
pub fn on_update() {
    let mut state = STATE.lock().unwrap();
    let state = state.get_or_insert_with(Default::default);

    // Sound, because the reference is dropped before we give back control to
    // the host.
    let buffer = unsafe { UPDATES.access() };

    let update = buffer.read_frame().to_vec();
    state.updates_from_runtime_tx.send(update).unwrap();
}
