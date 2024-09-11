use std::{fs::File, io::Read};

use anyhow::Result;
use clap::Parser;

use c8rs_core::Instruction;

#[derive(Parser, Debug)]
pub struct DisassemblerArgs {
    /// chip-8 ROM file
    file: String,

    #[arg(short = 'x')]
    /// show hexdump of file contents
    hexdump: bool,
}

pub fn disassemble(args: DisassemblerArgs) -> Result<()> {
    let file_contents = read_file(args.file)?;

    if args.hexdump {
        print_hexdump(file_contents);
    } else {
        print_disassembly(file_contents);
    }

    Ok(())
}

fn read_file(filename: String) -> Result<Vec<u8>> {
    let mut file = File::open(filename)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    Ok(buf)
}

fn print_hexdump(file_contents: Vec<u8>) {
    let chunks = file_contents.chunks(16);

    for (i, chunk) in chunks.enumerate() {
        let offset = i * 16;
        let chunk_str = chunk.iter().fold(String::new(), |mut acc, b| {
            acc.push_str(format!("{b:02X} ").as_str());
            acc
        });
        println!("|{offset:#06X}| {chunk_str}");
    }
}

fn print_disassembly(file_contents: Vec<u8>) {
    let mut offset = 0x200;

    for inst in file_contents.chunks(2) {
        let op = ((inst[0] as u16) << 8) | inst[1] as u16;

        let instr = Instruction::parse(op);
        println!("{offset:#06X}| {instr}",);

        offset += 2;
    }
}
