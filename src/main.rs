extern crate core;

use crate::script::{as_bool, as_script_nb, to_script_nb};

mod opcodes;
mod parse;
mod script;
mod interpret;


fn main() {
    let hex_script = "76a9149f21a07a0c7c3cf65a51f586051395762267cdaf88ac";
    let bin_script = hex::decode(hex_script).unwrap();
    let script = parse::parse_script(&bin_script).unwrap();

    println!("{:?}", &script);
    let bytes = to_script_nb(-0x50ab);
    println!("{}", hex::encode(&bytes));
    println!("{}", bytes.len());
    println!("{} -> {}", -0x50ab, as_script_nb(&bytes).unwrap());
    println!("{}", as_bool(&[0x80, 0x80]));
    println!("{}", hex::encode(to_script_nb(16)));
    let _ = interpret::interpret(&bin_script).unwrap();
}
