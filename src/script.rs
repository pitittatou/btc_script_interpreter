use std::fmt::{Debug, Formatter};
use crate::opcodes::Opcode as op;

#[derive(Debug)]
pub enum ScriptError {
    ParsingError
}

#[derive(Debug)]
pub struct Script<'a> {
    pub valid: bool,
    pub items: Vec<ScriptItem<'a>>
}

pub enum ScriptItem<'a> {
    Opcode(op),
    OpPushBytes,
    ByteArray(&'a [u8])
}

impl<'a> Debug for ScriptItem<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            ScriptItem::Opcode(op) => write!(f, "{}", op),
            ScriptItem::OpPushBytes => write!(f, "OP_PUSHBYTES"),
            ScriptItem::ByteArray(b) => write!(f, "{}", hex::encode(b))
        }
    }
}