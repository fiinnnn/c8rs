use std::{fs::File, io::Read};

use anyhow::Result;
use c8rs_core::Chip8Emulator;
use c8rs_disasm::DisassemblerArgs;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Run chip-8 emulator
    Run(RunArgs),

    /// Disassemble chip-8 binary
    #[command(visible_alias = "dis")]
    Disassemble(DisassemblerArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    file: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let res = match args.command {
        Command::Run(args) => run(args).await,
        Command::Disassemble(args) => disassemble(args),
    };

    if let Err(err) = res {
        println!("{err:?}");
    }
}

async fn run(args: RunArgs) -> Result<()> {
    let mut file = File::open(args.file)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let emu = Chip8Emulator::new(&buf);
    let controller = emu.controller();

    let mut app = c8rs_tui::App::new(controller);
    c8rs_tui::App::init_logger();

    emu.start();

    app.run().await
}

fn disassemble(args: DisassemblerArgs) -> Result<()> {
    c8rs_disasm::disassemble(args)
}
