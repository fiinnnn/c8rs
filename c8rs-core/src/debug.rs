use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Parser)]
#[command(name = "", multicall = true)]
pub enum DebugCommand {
    #[command(visible_alias = "s")]
    Step,

    #[command(visible_alias = "p")]
    Pause,

    #[command(visible_alias = "c")]
    Continue,

    #[command(name = "break", visible_alias = "b")]
    Breakpoint {
        #[clap(value_parser=clap_num::maybe_hex::<u16>)]
        addr: u16,
    },

    SetPc {
        #[clap(value_parser=clap_num::maybe_hex::<u16>)]
        addr: u16,
    },

    #[command(visible_alias = "rs")]
    Reset,

    #[command(name = "ips")]
    IPS { ips: u32 },
}

impl DebugCommand {
    pub fn parse_from(s: &str) -> Result<DebugCommand, String> {
        let s = shlex::split(s).ok_or("Invalid quoting".to_owned())?;
        DebugCommand::try_parse_from(s).map_err(|err| err.to_string())
    }
}
