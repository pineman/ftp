//#![feature(bigint_helper_methods)]
//#![feature(array_chunks)]
//#![feature(iter_intersperse)]

use std::collections::VecDeque;

pub use debug_interface::{DebugCmd, DebugEvent, DebugInterface};
use lr35902::LR35902;
use memory::{Bus, Buttons};
use ppu::{Frame, Ppu, DOTS_IN_ONE_FRAME};
use timer::Timer;

pub mod bw;
pub mod debug_interface;
pub mod debugger;
pub mod lr35902;
pub mod memory;
pub mod ppu;
pub mod timer;

pub struct Gameboy {
    bus: Bus,
    cpu: LR35902,
    ppu: Ppu,
    timer: Timer,
}

impl Gameboy {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let bus = Bus::new();
        Self {
            bus: bus.clone(),
            cpu: LR35902::new(bus.clone()),
            ppu: Ppu::new(bus.clone()),
            timer: Timer::new(bus),
        }
    }

    pub fn boot_real(&mut self) {
        self.bus_mut().load_bootrom();
        // Bootrom will be unloaded when it finishes - one of the last instructions
        // is writing to the BANK register which will trigger unload_bootrom
    }

    /// Sets CPU and hardware registers to the values found in the DMG0 column in the tables at
    /// <https://gbdev.io/pandocs/Power_Up_Sequence.html#console-state-after-boot-rom-hand-off>
    pub fn boot_fake(&mut self) {
        // CPU registers
        self.cpu.set_af(0x0100);
        self.cpu.set_bc(0xff13);
        self.cpu.set_de(0x00c1);
        self.cpu.set_hl(0x8403);
        self.cpu.set_sp(0xfffe);
        self.cpu.set_pc(0x100); // This skips executing the bootrom

        // HW registers
        self.bus.write(0xFF00, 0xCF); // P1
        self.bus.write(0xFF01, 0x00); // SB
        self.bus.write(0xFF02, 0x7E); // SC
        self.bus.write(0xFF04, 0x18); // DIV
        self.bus.write(0xFF05, 0x00); // TIMA
        self.bus.write(0xFF06, 0x00); // TMA
        self.bus.write(0xFF07, 0xF8); // TAC
        self.bus.write(0xFF0F, 0xE1); // IF
        self.bus.write(0xFF10, 0x80); // NR10
        self.bus.write(0xFF11, 0xBF); // NR11
        self.bus.write(0xFF12, 0xF3); // NR12
        self.bus.write(0xFF13, 0xFF); // NR13
        self.bus.write(0xFF14, 0xBF); // NR14
        self.bus.write(0xFF16, 0x3F); // NR21
        self.bus.write(0xFF17, 0x00); // NR22
        self.bus.write(0xFF18, 0xFF); // NR23
        self.bus.write(0xFF19, 0xBF); // NR24
        self.bus.write(0xFF1A, 0x7F); // NR30
        self.bus.write(0xFF1B, 0xFF); // NR31
        self.bus.write(0xFF1C, 0x9F); // NR32
        self.bus.write(0xFF1D, 0xFF); // NR33
        self.bus.write(0xFF1E, 0xBF); // NR34
        self.bus.write(0xFF20, 0xFF); // NR41
        self.bus.write(0xFF21, 0x00); // NR42
        self.bus.write(0xFF22, 0x00); // NR43
        self.bus.write(0xFF23, 0xBF); // NR44
        self.bus.write(0xFF24, 0x77); // NR50
        self.bus.write(0xFF25, 0xF3); // NR51
        self.bus.write(0xFF26, 0xF1); // NR52
        self.bus.write(0xFF40, 0x91); // LCDC
        self.bus.write(0xFF41, 0x81); // STAT
        self.bus.write(0xFF42, 0x00); // SCY
        self.bus.write(0xFF43, 0x00); // SCX
        self.bus.write(0xFF44, 0x91); // LY
        self.bus.write(0xFF45, 0x00); // LYC
        self.bus.write(0xFF46, 0xFF); // DMA
        self.bus.write(0xFF47, 0xFC); // BGP
        self.bus.write(0xFF48, 0x00); // OBP0
        self.bus.write(0xFF49, 0x00); // OBP1
        self.bus.write(0xFF4A, 0x00); // WY
        self.bus.write(0xFF4B, 0x00); // WX
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.bus.load_rom(rom);
    }

    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    pub fn bus_mut(&mut self) -> &mut Bus {
        &mut self.bus
    }

    pub fn cpu(&self) -> &LR35902 {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut LR35902 {
        &mut self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    pub fn step(&mut self) -> u8 {
        let cycles = self.cpu.step();
        // TODO: care for double speed mode (need to run half as much dots)
        self.ppu.step(cycles as u32);
        self.timer.step(self.cpu.clock_cycles());
        cycles
    }

    pub fn instruction(&mut self) -> u32 {
        let cycles = self.cpu.instruction() as u32;
        // TODO: care for double speed mode (need to run half as much dots)
        self.ppu.step(cycles);
        self.timer.step(self.cpu.clock_cycles());
        cycles
    }

    pub fn debug_cmd(&mut self, cmd: &DebugCmd) -> Option<DebugEvent> {
        self.cpu_mut().receive_command(cmd)
    }

    pub fn paused(&self) -> bool {
        self.cpu().paused()
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.cpu_mut().set_paused(paused)
    }

    pub fn get_debug_events(&mut self) -> &mut VecDeque<DebugEvent> {
        self.cpu.get_debug_events()
    }

    pub fn get_frame(&self) -> &Frame {
        self.ppu.get_frame()
    }

    pub fn cycles_in_one_frame(&self) -> u32 {
        // TODO: care for double speed mode
        DOTS_IN_ONE_FRAME
    }

    pub fn set_buttons(&mut self, buttons: &Buttons) {
        self.bus_mut().set_buttons(buttons)
    }
}
