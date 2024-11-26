//#![feature(array_chunks)]
//#![feature(iter_intersperse)]

use std::fs;

use clap::{Args, Parser, Subcommand, ValueEnum};
use fpt::debug_interface::{DebugCmd, DebugEvent};
use fpt::Gameboy;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    gameboy_config: GameboyConfig,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Args)]
struct GameboyConfig {
    /// Apply known CPU and hardware register values of a well-known bootrom when it
    /// hands off the execution to the cartridge's code. This skips emulating a bootrom.
    #[arg(short, long)]
    fake_bootrom: Option<BootromToFake>,
}

impl GameboyConfig {
    /// Build a `Gameboy` following this configuration. Consumes self.
    pub fn build_gameboy(self) -> Gameboy {
        let mut gameboy = Gameboy::new();
        if let Some(BootromToFake::DMG0) = self.fake_bootrom {
            gameboy.boot_fake();
        } else {
            gameboy.boot_real();
        }
        gameboy
    }
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
enum BootromToFake {
    DMG0,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Debug(Run),
    Dump(Dump),
    Run(Run),
}

#[derive(Debug, Args)]
struct Dump {
    rom: String,
}

#[derive(Debug, Args)]
struct Run {
    rom: String,
    #[arg(short, long)]
    debug: Option<bool>,
}

fn debug(args: Run) -> Result<()> {
    let mut gameboy = Gameboy::new();
    let rom = fs::read(args.rom).unwrap();
    gameboy.load_rom(&rom);

    let mut rl = DefaultEditor::new()?;
    if rl.load_history(".fpt_debug_history").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;

                let debug_cmd = DebugCmd::from_string(&line);
                if debug_cmd.is_none() {
                    println!("Error: cannot parse debug command");
                    continue;
                }
                let debug_event = gameboy.debug_cmd(&debug_cmd.unwrap());
                if debug_event.is_none() {
                    continue;
                }
                let debug_event = debug_event.unwrap();
                print!("{}", debug_event);

                if debug_event == DebugEvent::Continue {
                    loop {
                        if gameboy.paused() {
                            break;
                        }
                        gameboy.step();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history(".fpt_debug_history")?;
    Ok(())
}

fn dump(args: Dump) -> Result<()> {
    let mut gb = Gameboy::new();
    let rom = fs::read(args.rom).unwrap();
    gb.load_rom(&rom);
    loop {
        let inst = gb.cpu().decode();
        let result: Vec<String> = (1..inst.size)
            .map(|i| format!("{:#02X}", gb.cpu().mem8(gb.cpu().pc() + i as u16)))
            .collect();
        println!(
            "{:#02X}: {:?} ({:#02X}{}{})",
            gb.cpu().pc(),
            inst,
            inst.opcode,
            if result.is_empty() { "" } else { " " },
            result.join(" ")
        );
        // TODO: this is very, very stupid as it doesn't follow jumps, so it can
        // read data as code. how do decompilers even work?
        let next_pc = gb.cpu().pc() + inst.size as u16;
        gb.cpu_mut().set_pc(next_pc);
    }
}

fn run(gb_config: GameboyConfig, args: Run) -> Result<()> {
    let mut gameboy = gb_config.build_gameboy();

    let rom = fs::read(args.rom)?;
    gameboy.load_rom(&rom);
    loop {
        if args.debug.unwrap_or(false) {
            println!("{:#02X}: {:?}", gameboy.cpu().pc(), gameboy.cpu().decode());
        }
        gameboy.step();
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let gb_config = args.gameboy_config;

    match args.command {
        Commands::Debug(args) => debug(args),
        Commands::Dump(args) => dump(args),
        Commands::Run(args) => run(gb_config, args),
    }
}
