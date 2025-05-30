use std::time::{SystemTime, UNIX_EPOCH};

use lox_interpreter::{Interpreter, Value};

// standard library injection
// for now there is only clock()
pub fn set_stdlib(interpreter: &mut Interpreter) {
    interpreter.set_global_fn("clock", 0, |_args| {
        Value::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f32(),
        )
    });
}