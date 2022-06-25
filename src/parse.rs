use crate::opcodes::{OPCODES, OP_PUSH_DATA_1, OP_PUSH_DATA_2, OP_PUSH_DATA_4};
use crate::script::{Script, ScriptError};
use crate::script::ScriptError::ParsingErr;
use crate::script::ScriptItem::{ByteArray, Opcode};

pub fn parse(bytes: &[u8]) -> Result<Script, ScriptError> {
    let mut script = Vec::new();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let opcode = OPCODES.get(&bytes[cursor]).unwrap().clone();

        if opcode == OP_PUSH_DATA_1 || opcode == OP_PUSH_DATA_2 || opcode == OP_PUSH_DATA_4 {
            cursor += 1;
            let byte_nb = match opcode {
                OP_PUSH_DATA_1 => {
                    cursor += 1;
                    *bytes.get(cursor - 1).ok_or(ParsingErr)? as usize
                },
                OP_PUSH_DATA_2 => {
                    cursor += 2;
                    usize::from_le_bytes(bytes.get(cursor - 2..cursor).ok_or(ParsingErr)?.try_into().unwrap())
                },
                OP_PUSH_DATA_4 => {
                    cursor += 4;
                    usize::from_le_bytes(bytes.get(cursor - 4..cursor).ok_or(ParsingErr)?.try_into().unwrap())
                },
                _ => 0
            };

            let data = bytes.get(cursor..cursor + byte_nb).ok_or(ParsingErr)?;
            script.push(ByteArray(Vec::from(data)));
            cursor += byte_nb - 1;
        }
        // OP_PUSH_BYTES_X opcode
        else if opcode.code >= 1 && opcode.code <= 75 {
            let byte_nb = opcode.code as usize;
            cursor += 1;
            let data = bytes.get(cursor..cursor + byte_nb).ok_or(ParsingErr)?;
            script.push(ByteArray(Vec::from(data)));
            cursor += byte_nb - 1;
        }
        else {
            script.push(Opcode(opcode));
        }

        cursor += 1;
    }

    Ok(script)
}