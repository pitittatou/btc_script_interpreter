use std::cmp::{max, min};
use crate::{as_bool, as_script_nb};
use crate::script::{Script, SCRIPT_FALSE, SCRIPT_TRUE, ScriptError, ScriptItem, to_script_nb};
use crate::opcodes::*;

const MAX_STACK_SIZE: usize = 1000;
const MAX_SCRIPT_ELEMENT_SIZE: usize = 520;

pub struct Stack {
    pub main: Vec<Vec<u8>>,
    pub alt: Vec<Vec<u8>>
}

impl Stack {
    fn push(&mut self, bytes: Vec<u8>) -> Result<(), ScriptError> {
        if self.main.len() + self.alt.len() >= MAX_STACK_SIZE {
            return Err(ScriptError::StackOverflowErr)
        }
        if bytes.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(ScriptError::PushSizeErr)
        }
        self.main.push(bytes);
        Ok(())
    }

    fn push_alt(&mut self, bytes: Vec<u8>) -> Result<(), ScriptError> {
        if self.main.len() + self.alt.len() >= MAX_STACK_SIZE {
            return Err(ScriptError::StackOverflowErr)
        }
        if bytes.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(ScriptError::PushSizeErr)
        }
        self.alt.push(bytes);
        Ok(())
    }

    fn pop(&mut self) -> Result<Vec<u8>, ScriptError> {
        Ok(self.main.pop().ok_or(ScriptError::InvalidStackOperationErr)?)
    }

    fn pop_alt(&mut self) -> Result<Vec<u8>, ScriptError> {
        Ok(self.alt.pop().ok_or(ScriptError::InvalidAltStackOperationErr)?)
    }

    // Usage: stack.top(0) to get last element or stack.top(-1) to get 2nd element from the end
    fn top(&self, pos: i64) -> Result<Vec<u8>, ScriptError> {
        if pos > 0 {
            panic!("Wrong index given (positive)")
        }
        let idx = self.main.len() as i64 - 1 + pos;
        if idx < 0 {
            return Err(ScriptError::InvalidStackOperationErr)
        }
        Ok(self.main.get(idx as usize).ok_or(ScriptError::InvalidStackOperationErr)?.to_vec())
    }

    fn rm_top(&mut self, pos: i64) -> Result<Vec<u8>, ScriptError> {
        if pos > 0 {
            panic!("Wrong index given (positive)")
        }
        let idx = self.main.len() as i64 - 1 + pos;
        if idx < 0 {
            return Err(ScriptError::InvalidStackOperationErr)
        }
        Ok(self.main.remove(idx as usize))
    }

    fn swap_top(&mut self, a: i64, b: i64) -> Result<(), ScriptError> {
        if a > 0 || b > 0 {
            panic!("Wrong index given (positive)")
        }
        let idx_a = self.main.len() as i64 - 1 + a;
        let idx_b = self.main.len() as i64 - 1 + b;
        if idx_a < 0 || idx_b < 0 {
            return Err(ScriptError::InvalidStackOperationErr)
        }
        Ok(self.main.swap(idx_a as usize, idx_b as usize))
    }
}

pub fn interpret(script: &Script) -> Result<bool, ScriptError>{
    let mut stack = Stack {main: Vec::with_capacity(20), alt: Vec::with_capacity(20)};
    let mut conditional_tree: Vec<Option<bool>> = Vec::with_capacity(10);
    let mut execute = true;

    for item in script {
        match item {
            ScriptItem::ByteArray(b) => {
                if b.len() > MAX_SCRIPT_ELEMENT_SIZE {
                    return Err(ScriptError::PushSizeErr)
                }
                if execute {
                    stack.push(b.to_owned())?
                }
            },
            ScriptItem::Opcode(op) => {
                if DISABLED_OPCODES.contains(op) {
                    return Err(ScriptError::DisabledOpcodeErr)
                }
                if *op == OP_VERIF || *op == OP_VERNOTIF {
                    return Err(ScriptError::InvalidOpcodeErr)
                }
                if execute || (OP_IF.code <= op.code && op.code <= OP_ENDIF.code) {
                    match *op {
                        //
                        // Constants
                        //
                        OP_0 => stack.push(to_script_nb(0))?,
                        OP_1NEGATE => stack.push(to_script_nb(-1))?,
                        Opcode { code: c } if c >= OP_1.code && c <= OP_16.code => stack.push(to_script_nb((c - OP_1.code + 1) as i64))?,

                        //
                        // Flow Control
                        //
                        OP_NOP => {}
                        OP_IF | OP_NOTIF => {
                            conditional_tree.push(None);
                            if execute {
                                let mut condition = as_bool(&stack.pop()?);
                                if *op == OP_NOTIF {
                                    condition = !condition;
                                }
                                *conditional_tree.last_mut().unwrap() = Some(condition);
                                execute = condition;
                            }
                        }
                        OP_ELSE => {
                            if conditional_tree.is_empty() {
                                return Err(ScriptError::UnbalancedConditionalErr)
                            }
                            let last = conditional_tree.last_mut().unwrap();
                            match *last {
                                Some(state) => {
                                    *last = Some(!state);
                                    execute = !state;
                                },
                                None => {}
                            }
                        }
                        OP_ENDIF => {
                            if conditional_tree.is_empty() {
                                return Err(ScriptError::UnbalancedConditionalErr)
                            }
                            conditional_tree.pop();
                            if conditional_tree.is_empty() {
                                execute = true
                            } else {
                                execute = conditional_tree.last().unwrap().unwrap()
                            }
                        }
                        OP_VERIFY => {
                            let v = as_bool(&stack.pop()?);
                            if !v {
                                return Err(ScriptError::VerifyErr)
                            }
                        }
                        OP_RETURN => return Err(ScriptError::OpReturnErr),

                        //
                        // Stack
                        //
                        OP_TOALTSTACK => {
                            let v = stack.pop()?;
                            stack.push_alt(v)?
                        }
                        OP_FROMALTSTACK => {
                            let v = stack.pop_alt()?;
                            stack.push(v)?
                        }
                        OP_2DROP => {
                            stack.pop()?;
                            stack.pop()?;
                        }
                        OP_2DUP => {
                            let v1 = stack.top(-1)?;
                            let v2 = stack.top(0)?;
                            stack.push(v1)?;
                            stack.push(v2)?
                        }
                        OP_3DUP => {
                            let v1 = stack.top(-2)?;
                            let v2 = stack.top(-1)?;
                            let v3 = stack.top(0)?;
                            stack.push(v1)?;
                            stack.push(v2)?;
                            stack.push(v3)?
                        }
                        OP_2OVER => {
                            let v1 = stack.top(-3)?;
                            let v2 = stack.top(-2)?;
                            stack.push(v1)?;
                            stack.push(v2)?
                        }
                        OP_2ROT => {
                            let v1 = stack.rm_top(-5)?;
                            let v2 = stack.rm_top(-4)?;
                            stack.push(v1)?;
                            stack.push(v2)?
                        }
                        OP_2SWAP => {
                            stack.swap_top(0, -2)?;
                            stack.swap_top(-1, -3)?
                        }
                        OP_IFDUP => {
                            let v = stack.top(0)?;
                            if as_bool(&v) {
                                stack.push(v)?
                            }
                        }
                        OP_DEPTH => {
                            let v = to_script_nb(stack.main.len() as i64);
                            stack.push(v)?
                        }
                        OP_DROP => { stack.pop()?; }
                        OP_DUP => {
                            let v = stack.top(0)?;
                            stack.push(v)?
                        }
                        OP_NIP => { stack.rm_top(-1)?; }
                        OP_OVER => {
                            let v = stack.top(-1)?;
                            stack.push(v)?
                        }
                        OP_PICK => {
                            let n = as_script_nb(&stack.pop()?)?;
                            let v = stack.top(-n)?;
                            stack.push(v)?
                        }
                        OP_ROLL => {
                            let n = as_script_nb(&stack.pop()?)?;
                            let v = stack.rm_top(-n)?;
                            stack.push(v)?
                        }
                        OP_ROT => {
                            let v = stack.rm_top(-2)?;
                            stack.push(v)?
                        }
                        OP_SWAP => stack.swap_top(0, -1)?,
                        OP_TUCK => {
                            let v1 = stack.pop()?;
                            let v2 = stack.pop()?;
                            let v3 = v1.clone();
                            stack.push(v1)?;
                            stack.push(v2)?;
                            stack.push(v3)?
                        }

                        //
                        // Splice
                        //
                        OP_SIZE => {
                            let v = to_script_nb(stack.top(0)?.len() as i64);
                            stack.push(v)?
                        }

                        //
                        // Bitwise Logic
                        //
                        OP_EQUAL | OP_EQUALVERIFY => {
                            let v1 = stack.pop()?;
                            let v2 = stack.pop()?;
                            if v1 == v2 {
                                stack.push(Vec::from(SCRIPT_TRUE))?
                            } else {
                                stack.push(Vec::from(SCRIPT_FALSE))?
                            }

                            if *op == OP_EQUALVERIFY {
                                if v1 == v2 {
                                    stack.pop()?;
                                } else {
                                    return Err(ScriptError::EqualVerifyErr)
                                }
                            }
                        }

                        //
                        // Arithmetic
                        //
                        OP_1ADD | OP_1SUB | OP_NEGATE | OP_ABS | OP_NOT | OP_0NOTEQUAL => {
                            let mut v = as_script_nb(&stack.pop()?)?;
                            match *op {
                                OP_1ADD => v += 1,
                                OP_1SUB => v -= 1,
                                OP_NEGATE => v *= -1,
                                OP_ABS => v = v.abs(),
                                OP_NOT => v = (v == 0) as i64,
                                OP_0NOTEQUAL => v = (v != 0) as i64,
                                _ => {}
                            }
                            stack.push(to_script_nb(v))?
                        }
                        OP_ADD | OP_SUB | OP_BOOLAND | OP_BOOLOR | OP_NUMEQUAL | OP_NUMEQUALVERIFY |
                        OP_NUMNOTEQUAL | OP_LESSTHAN | OP_GREATERTHAN | OP_LESSTHANOREQUAL |
                        OP_GREATERTHANOREQUAL | OP_MIN | OP_MAX => {
                            let v2 = as_script_nb(&stack.pop()?)?;
                            let v1 = as_script_nb(&stack.pop()?)?;
                            let res = match *op {
                                OP_ADD => v1 + v2,
                                OP_SUB => v1 - v2,
                                OP_BOOLAND => (v1 != 0 && v2 != 0) as i64,
                                OP_BOOLOR => (v1 != 0 || v2 != 0) as i64,
                                OP_NUMEQUAL | OP_NUMEQUALVERIFY => (v1 == v2) as i64,
                                OP_NUMNOTEQUAL => (v1 != v2) as i64,
                                OP_LESSTHAN => (v1 < v2) as i64,
                                OP_GREATERTHAN => (v1 > v2) as i64,
                                OP_LESSTHANOREQUAL => (v1 <= v2) as i64,
                                OP_GREATERTHANOREQUAL => (v1 >= v2) as i64,
                                OP_MIN => min(v1, v2),
                                OP_MAX => max(v1, v2),
                                _ => 0
                            };
                            stack.push(to_script_nb(res))?;

                            if *op == OP_NUMEQUALVERIFY {
                                if v1 == v2 {
                                    stack.pop()?;
                                } else {
                                    return Err(ScriptError::NumEqualVerifyErr)
                                }
                            }
                        }
                        OP_WITHIN => {
                            let max = stack.pop()?;
                            let min = stack.pop()?;
                            let x = stack.pop()?;
                            let res = (min <= x && x < max) as i64;
                            stack.push(to_script_nb(res))?
                        }

                        //
                        // Crypto
                        //

                        _ => return Err(ScriptError::BadOpcodeErr)
                    }
                }
            }
        }
        if !conditional_tree.is_empty() {
            return Err(ScriptError::UnbalancedConditionalErr)
        }
    }

    Ok(true)
}