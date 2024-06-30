use fpt::{DebugCmd, DebugEvent, Gameboy};

fn check_registers(gb: &Gameboy) -> bool {
    return gb.cpu().b() == 3
        && gb.cpu().c() == 5
        && gb.cpu().d() == 8
        && gb.cpu().e() == 13
        && gb.cpu().h() == 21
        && gb.cpu().l() == 34;
}

fn rom_test(rom_path: &str, termination_address: u16) {
    let mut gb = Gameboy::new();
    gb.simulate_dmg0_bootrom_handoff_state();

    let rom = std::fs::read(rom_path).unwrap();
    gb.load_rom(&rom);

    gb.debug_cmd(&DebugCmd::Instrpoint(0x40));
    gb.debug_cmd(&DebugCmd::Breakpoint(termination_address));

    'outer: loop {
        gb.step();
        let debug_events = gb.get_debug_events();
        if !debug_events.is_empty() {
            loop {
                match debug_events.pop_back().unwrap() {
                    DebugEvent::Breakpoint(_) => {
                        println!("breakpoint");
                        break 'outer;
                    }
                    DebugEvent::Instrpoint(_) => {
                        println!("instrpoint");
                        assert!(check_registers(&gb) == true);
                        continue 'outer;
                    }
                    _ => continue 'outer,
                }
            }
        }
    }
}

#[test]
fn rom_test1() {
    rom_test(
        "../third_party/mooneye-test-suite/build/acceptance/timer/tim00.gb",
        0x4ab4,
    );
}

#[test]
fn rom_test2() {
    rom_test(
        "../third_party/mooneye-test-suite/build/acceptance/timer/tim01.gb",
        0x4ab4,
    );
}
