use crate::opcodes::{OPCODES, OP_PUSH_DATA_1, OP_PUSH_DATA_2, OP_PUSH_DATA_4};
use crate::script::{Script, ScriptError, ScriptItem};
use crate::script::ScriptItem::{ByteArray, Opcode};

pub fn parse_script(bytes: &[u8]) -> Result<Script, ScriptError> {
    let mut script = Vec::new();
    let mut cursor: usize = 0;
    let mut item = parse_one_op(bytes, &mut cursor)?;

    while item.is_some() {
        script.push(item.unwrap());
        item = parse_one_op(bytes, &mut cursor)?;
    }

    Ok(script)
}

pub fn parse_one_op(bytes: &[u8], pc: &mut usize) -> Result<Option<ScriptItem>, ScriptError> {
    if *pc >= bytes.len() {
        return Ok(None)
    }

    let opcode = OPCODES.get(&bytes[*pc]).unwrap().clone();
    if opcode == OP_PUSH_DATA_1 || opcode == OP_PUSH_DATA_2 || opcode == OP_PUSH_DATA_4 {
        *pc += 1;
        let byte_nb = match opcode {
            OP_PUSH_DATA_1 => {
                *pc += 1;
                *bytes.get(*pc - 1).ok_or(ScriptError::BadOpcodeErr)? as usize
            },
            OP_PUSH_DATA_2 => {
                *pc += 2;
                usize::from_le_bytes(bytes.get(*pc - 2..*pc).ok_or(ScriptError::BadOpcodeErr)?.try_into().unwrap())
            },
            OP_PUSH_DATA_4 => {
                *pc += 4;
                usize::from_le_bytes(bytes.get(*pc - 4..*pc).ok_or(ScriptError::BadOpcodeErr)?.try_into().unwrap())
            },
            _ => 0
        };

        let data = bytes.get(*pc..*pc + byte_nb).ok_or(ScriptError::BadOpcodeErr)?;
        *pc += byte_nb;
        Ok(Some(ByteArray(Vec::from(data))))
    }
    // OP_PUSH_BYTES_X opcode
    else if opcode.code >= 1 && opcode.code <= 75 {
        let byte_nb = opcode.code as usize;
        *pc += 1;
        let data = bytes.get(*pc..*pc + byte_nb).ok_or(ScriptError::BadOpcodeErr)?;
        *pc += byte_nb;
        Ok(Some(ByteArray(Vec::from(data))))
    } else {
        *pc += 1;
        Ok(Some(Opcode(opcode)))
    }
}