extern crate core;

mod opcodes;
mod parse;
mod script;


fn main() {
    let hex_script = "76a9149f21a07a0c7c3cf65a51f586051395762267cdaf88ac";
    let bin_script = hex::decode(hex_script).unwrap();
    let script = parse::parse(&bin_script, false).unwrap();

    println!("{:?}", &script);
}
