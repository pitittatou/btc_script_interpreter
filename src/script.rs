use std::fmt::{Debug, Formatter};
use crate::opcodes::Opcode as op;

pub const MAX_NUM_SIZE: usize = 4;

#[derive(Debug)]
pub enum ScriptError {
    ParsingError,
    ScriptNumberOverflow
}

#[derive(Debug)]
pub struct Script {
    pub valid: bool,
    pub items: Vec<ScriptItem>
}

// Not sure if I should use references that would probably be more optimized but create code bloat
pub enum ScriptItem {
    Opcode(op),
    ByteArray(Vec<u8>)
}

impl Debug for ScriptItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptItem::Opcode(op) => write!(f, "{}", op),
            ScriptItem::ByteArray(b) => write!(f, "{}", hex::encode(b))
        }
    }
}

// Convert an int to the Script Number format used on the stack
pub fn to_script_nb(value: i64) -> Vec<u8> {
    let mut result = Vec::with_capacity(4);
    if value == 0 {
        return result
    }

    let neg = value < 0;
    let mut abs_value = value.abs();
    while abs_value > 0 {
        result.push((abs_value & 0xff) as u8);
        abs_value >>= 8;
    }

    if result.last().unwrap() & 0x80 != 0 {
        result.push(match neg { true => 0x80, false => 0 });
    } else if neg {
        *result.last_mut().unwrap() |= 0x80;
    }

    return result
}

// Convert a Script Number to an int
// Only numbers of at most 4 bytes are accepted
pub fn as_script_nb(bytes: &[u8]) -> Result<i64, ScriptError> {
    if bytes.len() > MAX_NUM_SIZE {
        return Err(ScriptError::ScriptNumberOverflow)
    }

    if bytes.is_empty() {
        return Ok(0)
    }

    let mut result= 0;
    for i in 0..bytes.len() {
        result |= (bytes[i] as i64) << 8*i;
    }

    if bytes.last().unwrap() & 0x80 != 0 {
        return Ok(-(result & !(0x80u64 << (8 * (bytes.len() - 1))) as i64))
    }

    Ok(result)
}

pub fn as_bool(bytes: &[u8]) -> bool {
    return !(bytes.len() == 0 || bytes == [0x80])
}