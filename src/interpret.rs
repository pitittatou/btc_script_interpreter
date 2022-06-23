use crate::script::{Script, ScriptError, ScriptItem};

const MAX_STACK_SIZE: usize = 1000;

pub struct Stacks<'a> {
    pub main: Vec<&'a [u8]>,
    pub alt: Vec<&'a [u8]>
}

pub fn interpret(script: &Script) -> Result<bool, ScriptError>{
    let mut stacks = Stacks {main: Vec::with_capacity(MAX_STACK_SIZE), alt: Vec::with_capacity(MAX_STACK_SIZE)};

    for item in &script.items {
        match item {
            ScriptItem::ByteArray(b) => stacks.main.push(b),
            _ => {}
        }
    }

    Ok(true)
}