use std::collections::VecDeque;
use std::fmt;

use instructions::{Instruction, InstructionKind, INSTRUCTIONS};

use super::memory::Bus;
use crate::debug_interface::{DebugCmd, DebugEvent, DebugInterface};
use crate::debugger::Debugger;
use crate::ppu::Mode;
use crate::{bw, memory};

pub mod instructions;

fn carrying_add(x:u8, y:u8, carry: bool) -> (u8, bool) {
    let carry: u8 = carry as u8;
    let half_adder = x ^ y ;
    let sum = half_adder ^ carry;
    let carry = (carry & half_adder) | (x & y);
    (sum, carry == 1)
}

// yolo
fn borrowing_sub(x:u8, y:u8, borrow:bool) -> (u8, bool) {
    carrying_add(x, !y, !borrow)
}

#[derive(Clone, PartialEq)]
pub struct LR35902 {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    ime: bool,
    ime_next_inst: bool,
    prefix_cb: bool,
    clock_cycles: u64,
    inst_cycle_count: u8,
    branch_taken: bool,
    halted: bool,
    bus: Bus,
    debugger: Debugger,
}

impl Default for LR35902 {
    fn default() -> Self {
        Self::new(Bus::new())
    }
}

impl fmt::Debug for LR35902 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LR35902 {{ a: {:#04X}, f: {:#06b}, bc: {:#06X}, de: {:#06X}, hl: {:#06X}, sp: {:#06X}, pc: {:#06X}, clock_cycles: {} }} ", self.a(), self.f() >> 4, self.bc, self.de, self.hl, self.sp, self.pc, self.clock_cycles)
    }
}

impl DebugInterface for LR35902 {
    fn receive_command(&mut self, cmd: &DebugCmd) -> Option<DebugEvent> {
        self.debugger.receive_command(cmd)
    }

    fn paused(&self) -> bool {
        self.debugger.paused
    }

    fn set_paused(&mut self, paused: bool) {
        self.debugger.paused = paused;
    }

    fn get_debug_events(&mut self) -> &mut VecDeque<DebugEvent> {
        self.debugger.debug_events()
    }
}

impl LR35902 {
    pub fn new(bus: Bus) -> Self {
        Self {
            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0,
            pc: 0,
            ime: false,
            ime_next_inst: false,
            prefix_cb: false,
            clock_cycles: 0,
            inst_cycle_count: 0,
            branch_taken: false,
            halted: false,
            bus: bus.clone(),
            // Debugging
            debugger: Debugger::new(bus.clone()),
        }
    }

    // Registers
    pub fn a(&self) -> u8 {
        bw::get_byte16::<1>(self.af)
    }

    pub fn set_a(&mut self, value: u8) {
        self.af = bw::set_byte16::<1>(self.af, value);
    }

    pub fn f(&self) -> u8 {
        bw::get_byte16::<0>(self.af)
    }

    pub fn set_f(&mut self, value: u8) {
        self.af = bw::set_byte16::<0>(self.af, value);
    }

    pub fn af(&self) -> u16 {
        self.af
    }

    pub fn set_af(&mut self, af: u16) {
        self.af = af;
    }

    pub fn b(&self) -> u8 {
        bw::get_byte16::<1>(self.bc)
    }

    pub fn set_b(&mut self, value: u8) {
        self.bc = bw::set_byte16::<1>(self.bc, value);
    }

    pub fn c(&self) -> u8 {
        bw::get_byte16::<0>(self.bc)
    }

    pub fn set_c(&mut self, value: u8) {
        self.bc = bw::set_byte16::<0>(self.bc, value);
    }

    pub fn bc(&self) -> u16 {
        self.bc
    }

    pub fn set_bc(&mut self, bc: u16) {
        self.bc = bc;
    }

    pub fn d(&self) -> u8 {
        bw::get_byte16::<1>(self.de)
    }

    pub fn set_d(&mut self, value: u8) {
        self.de = bw::set_byte16::<1>(self.de, value);
    }

    pub fn e(&self) -> u8 {
        bw::get_byte16::<0>(self.de)
    }

    pub fn set_e(&mut self, value: u8) {
        self.de = bw::set_byte16::<0>(self.de, value);
    }

    pub fn de(&self) -> u16 {
        self.de
    }

    pub fn set_de(&mut self, de: u16) {
        self.de = de;
    }

    pub fn h(&self) -> u8 {
        bw::get_byte16::<1>(self.hl)
    }

    pub fn set_h(&mut self, value: u8) {
        self.hl = bw::set_byte16::<1>(self.hl, value);
    }

    pub fn l(&self) -> u8 {
        bw::get_byte16::<0>(self.hl)
    }

    pub fn set_l(&mut self, value: u8) {
        self.hl = bw::set_byte16::<0>(self.hl, value);
    }

    pub fn hl(&self) -> u16 {
        self.hl
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.hl = hl;
    }

    pub fn sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, sp: u16) {
        self.sp = sp;
    }

    // flags
    pub fn z_flag(&self) -> bool {
        bw::test_bit16::<7>(self.af)
    }

    pub fn set_z_flag(&mut self, value: bool) {
        self.af = bw::set_bit16::<7>(self.af, value);
    }

    pub fn n_flag(&self) -> bool {
        bw::test_bit16::<6>(self.af)
    }

    pub fn set_n_flag(&mut self, value: bool) {
        self.af = bw::set_bit16::<6>(self.af, value);
    }

    pub fn h_flag(&self) -> bool {
        bw::test_bit16::<5>(self.af)
    }

    pub fn set_h_flag(&mut self, value: bool) {
        self.af = bw::set_bit16::<5>(self.af, value);
    }

    pub fn c_flag(&self) -> bool {
        bw::test_bit16::<4>(self.af)
    }

    pub fn set_c_flag(&mut self, value: bool) {
        self.af = bw::set_bit16::<4>(self.af, value);
    }

    // Internal state
    pub fn pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    pub fn interrupt_master_enable(&self) -> bool {
        self.ime
    }

    pub fn set_ime(&mut self, ime: bool) {
        self.ime = ime;
    }

    pub fn set_ime_next_inst(&mut self) {
        self.ime_next_inst = true;
    }

    pub fn clock_cycles(&self) -> u64 {
        self.clock_cycles
    }

    pub fn set_clock_cycles(&mut self, clock_cycles: u64) {
        self.clock_cycles = clock_cycles;
    }

    pub fn mutated_pc(&self) -> bool {
        self.branch_taken
    }

    pub fn set_mutated_pc(&mut self, branch_taken: bool) {
        self.branch_taken = branch_taken;
    }

    pub fn inst_cycle_count(&self) -> u8 {
        self.inst_cycle_count
    }

    pub fn set_inst_cycle_count(&mut self, inst_cycle_count: u8) {
        self.inst_cycle_count = inst_cycle_count;
    }

    // Memory
    pub fn mem8(&self, index: u16) -> u8 {
        // TODO: watchpoint trigger read
        self.bus.read(index as usize)
    }

    pub fn mem16(&self, index: u16) -> u16 {
        bw::word16(self.mem8(index + 1), self.mem8(index))
    }

    pub fn set_mem8(&mut self, index: u16, value: u8) {
        self.bus.write(index as usize, value);
        // TODO: watchpoint trigger write
        // Write triggers (TODO: better solution)
        if (index == memory::map::BANK as u16) && (value != 0) {
            self.bus.unload_bootrom();
        }
        if index == memory::map::LCDC as u16
            && bw::test_bit8::<7>(self.bus.lcdc())
            && !bw::test_bit8::<7>(value)
            && self.bus.stat() & 0b00000011 != Mode::VBlank as u8
        {
            panic!("tried to change lcdc.7 when ppu is not in vblank!");
        }
    }

    pub fn set_mem16(&mut self, index: u16, value: u16) {
        self.set_mem8(index + 1, bw::get_byte16::<1>(value));
        self.set_mem8(index, bw::get_byte16::<0>(value));
    }

    // Decoding
    /// get 8 bit immediate at position pc + 1 + pos
    fn get_d8(&self, pos: u8) -> u8 {
        self.mem8(self.pc + pos as u16 + 1)
    }

    /// get 8 bit immediate at position pc + 1 + pos
    fn get_r8(&self, pos: u8) -> i8 {
        self.mem8(self.pc + pos as u16 + 1) as i8
    }

    /// get 16 bit immediate at position pc + 1 + pos
    fn get_d16(&self, pos: u8) -> u16 {
        // little-endian: the first byte in memory is the LSB
        ((self.get_d8(pos + 1) as u16) << 8) + self.get_d8(pos) as u16
    }

    fn hl_ind(&self) -> u8 {
        self.mem8(self.hl())
    }

    fn set_hl_ind(&mut self, value: u8) {
        self.set_mem8(self.hl(), value);
    }

    pub fn decode(&self) -> Instruction {
        let mut opcode = self.mem8(self.pc()) as u16;
        if self.prefix_cb {
            opcode += 0x100;
        }
        INSTRUCTIONS[opcode as usize]
    }

    pub fn decode_ahead(&self, n: usize) -> Vec<(u16, Instruction)> {
        let mut ret = Vec::<(u16, Instruction)>::with_capacity(n + 1);
        ret.push((self.pc(), self.decode()));
        // TODO: decode can return "PREFIX CB" - not good for the disasm
        //  if ret[0].mnemonic == "PREFIX CB" {}
        for i in 1..(n + 1) {
            let last_inst = ret[i - 1];
            let next_pc = last_inst.0 + last_inst.1.size as u16;
            let mut next_inst_opcode_index = self.mem8(next_pc) as usize;
            if last_inst.1.mnemonic == "PREFIX CB" {
                next_inst_opcode_index += 0x100;
            }
            ret.push((next_pc, INSTRUCTIONS[next_inst_opcode_index]));
        }
        ret
    }

    // Instruction logic
    fn half_carry8(&self, x: u8, y: u8) -> bool {
        ((x & 0x0f) + (y & 0x0f)) > 0x0f
    }

    fn half_carryc8(&self, x: u8, y: u8, c: u8) -> bool {
        ((x & 0x0f) + (y & 0x0f) + c) > 0x0f
    }

    fn half_carry16(&self, x: u16, y: u16) -> bool {
        ((x & 0x0fff) + (y & 0x0fff)) > 0x0fff
    }

    fn half_carry16i(&self, x: u16, y: i8) -> bool {
        (x & 0x0fff).wrapping_add_signed(y as i16) > 0x0fff
    }

    fn inc8(&mut self, x: u8) -> u8 {
        let (result, _overflow) = x.overflowing_add(1);
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(self.half_carry8(x, 1));
        // INC r8 instructions don't set the C (carry) flag
        result
    }

    fn dec8(&mut self, x: u8) -> u8 {
        let (result, _overflow) = x.overflowing_sub(1);
        self.set_z_flag(result == 0);
        self.set_n_flag(true);
        // There was a carry in bit 3 if the result's least significant nibble
        // is all 0s (should we use a generalization to the half-carry logic?)
        self.set_h_flag(result & 0xF == 0);
        // DEC r8 instructions don't set the C (carry) flag
        result
    }

    fn inc16(&mut self, x: u16) -> u16 {
        let (result, _overflow) = x.overflowing_add(1);
        // No flags affected
        result
    }

    fn dec16(&mut self, x: u16) -> u16 {
        let (result, _overflow) = x.overflowing_sub(1);
        // No flags affected
        result
    }

    fn add8(&mut self, x: u8, y: u8) -> u8 {
        let (result, overflow) = x.overflowing_add(y);
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(self.half_carry8(x, y));
        self.set_c_flag(overflow);
        result
    }
    
    fn addc8(&mut self, x: u8, y: u8) -> u8 {
        let (result, overflow) = carrying_add(x, y, self.c_flag());
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(self.half_carryc8(x, y, self.c_flag() as u8));
        self.set_c_flag(overflow);
        result
    }

    fn sub8(&mut self, x: u8, y: u8) -> u8 {
        // every day I'm grateful for overflowing_sub
        let (result, overflow) = x.overflowing_sub(y);
        self.set_z_flag(result == 0);
        self.set_n_flag(true);
        self.set_h_flag((x & 0x0f).overflowing_sub(y & 0x0f).1);
        self.set_c_flag(overflow);
        result
    }

    fn subc8(&mut self, x: u8, y: u8) -> u8 {
        let (result, overflow) = borrowing_sub(x, y, self.c_flag());
        self.set_z_flag(result == 0);
        self.set_n_flag(true);
        self.set_h_flag(borrowing_sub(x & 0x0f, y & 0x0f, self.c_flag()).1);
        self.set_c_flag(overflow);
        result
    }

    fn add16(&mut self, x: u16, y: u16) -> u16 {
        let (result, overflow) = x.overflowing_add(y);
        // z flag is not set
        self.set_n_flag(false);
        self.set_h_flag(self.half_carry16(x, y));
        self.set_c_flag(overflow);
        result
    }

    fn add16i(&mut self, x: u16, y: i8) -> u16 {
        let (result, overflow) = x.overflowing_add_signed(y as i16);
        self.set_z_flag(false);
        self.set_n_flag(false);
        self.set_h_flag(self.half_carry16i(x, y));
        self.set_c_flag(overflow);
        result
    }

    fn xor8(&mut self, x: u8) -> u8 {
        let result = self.a() ^ x;
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(false);
        result
    }

    fn and8(&mut self, x: u8) -> u8 {
        let result = self.a() & x;
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(true);
        self.set_c_flag(false);
        result
    }

    fn or8(&mut self, x: u8) -> u8 {
        let result = self.a() | x;
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(false);
        result
    }

    fn rlc8(&mut self, x: u8) -> u8 {
        let result = x.rotate_left(1);
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<7>(x));
        result
    }

    fn rl8(&mut self, x: u8) -> u8 {
        let c_result = x.rotate_left(1);
        let result = bw::set_bit8::<0>(c_result, self.c_flag());
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<0>(c_result));
        result
    }

    fn rrc8(&mut self, x: u8) -> u8 {
        let result = x.rotate_right(1);
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<0>(x));
        result
    }

    fn rr8(&mut self, x: u8) -> u8 {
        let c_result = x.rotate_right(1);
        let result = bw::set_bit8::<7>(c_result, self.c_flag());
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<7>(c_result));
        result
    }

    fn sla8(&mut self, x: u8) -> u8 {
        let result = x << 1;
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<7>(x));
        result
    }

    fn sra8(&mut self, x: u8) -> u8 {
        let msb_result = x >> 1;
        let result = bw::set_bit8::<7>(msb_result, bw::test_bit8::<7>(x));
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<0>(x));
        result
    }

    fn srl8(&mut self, x: u8) -> u8 {
        let result = x >> 1;
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(bw::test_bit8::<0>(x));
        result
    }

    fn swap8(&mut self, x: u8) -> u8 {
        let lo = x & 0x0f;
        let hi = x & 0xf0;
        let result = (lo << 4) + (hi >> 4);
        self.set_z_flag(result == 0);
        self.set_n_flag(false);
        self.set_h_flag(false);
        self.set_c_flag(false);
        result
    }

    fn push(&mut self, value: u16) {
        self.set_sp(self.sp() - 2);
        self.set_mem16(self.sp(), value);
    }

    fn pop(&mut self) -> u16 {
        let r = self.mem16(self.sp());
        self.set_sp(self.sp() + 2);
        r
    }

    fn calc_jr_address(&self, pc: u16, offset: i8) -> u16 {
        // pc + 2 because relative address are based off
        // the end of the JR instruction.
        let r = (pc + 2) as i32 + offset as i32;
        if !(0..=65535).contains(&r) {
            panic!();
        }

        r as u16
    }

    fn jump(&mut self, address: u16) {
        self.set_pc(address);
        self.set_mutated_pc(true);
    }

    fn call(&mut self, address: u16) {
        // pc + 3 because CALLs have size == 3 bytes
        self.push(self.pc() + 3);
        self.jump(address);
    }

    fn ret(&mut self) {
        let address = self.pop();
        self.jump(address);
    }

    fn rst(&mut self, address: u16) {
        // pc + 1 because RSTs have size == 1 byte
        self.push(self.pc() + 1);
        self.jump(address);
    }

    fn reti(&mut self) {
        // TODO: The interrupt master enable flag is returned to its pre-interrupt status.
        //  BUT: https://gbdev.io/pandocs/Interrupts.htm claims that RETI is EI followed by RET
        self.set_ime_next_inst();

        // RET
        let address = self.pop();
        self.jump(address);
    }

    fn bit<const INDEX: u8>(&mut self, x: u8) {
        self.set_z_flag(!bw::test_bit8::<INDEX>(x));
        self.set_n_flag(false);
        self.set_h_flag(true);
    }

    pub fn update_code_listing(&mut self, inst: Instruction) {
        if self.bus.memory().code_listing()[self.pc() as usize].is_some() {
            return;
        }
        let result: Vec<String> = (1..inst.size)
            .map(|i| format!("{:#02X}", self.mem8(self.pc() + i as u16)))
            .collect();
        let str = format!(
            "{:#06X}: {} ({:#02X}{}{})",
            self.pc(),
            inst.mnemonic,
            inst.opcode,
            if result.is_empty() { "" } else { " " },
            result.join(" ")
        );
        self.bus.memory_mut().set_code_listing_at(self.pc(), str);
    }

    // Run instructions
    /// Returns the number of t-cycles actually ran. Should be 1 t-cycle most of the time,
    /// but sometimes may be 1 + 5 cycles in case an interrupt was hit, or even 0 on breakpoints.
    /// This probably points to the fact that we should have a real clock() function
    /// that runs *one* clock cycle of the whole system. These step functions of
    /// each subsystem could be like coroutines for easier state tracking.
    /// But, for now, it's easier to run multiple cycles in each step().
    pub fn step(&mut self) -> u8 {
        let inst = self.decode();
        if !self.halted {
            // Only actually mutate CPU state on the last t-cycle of the instruction
            if self.inst_cycle_count() + 1 < inst.cycles {
                self.set_inst_cycle_count(self.inst_cycle_count() + 1);
                return 1;
            }

            self.update_code_listing(inst);
            if self.debugger.match_breakpoint(self.pc()) {
                return 0;
            }
            if self.debugger.match_instrpoint(inst.opcode) {
                return 0;
            }

            self.execute(inst);

            if !self.mutated_pc() {
                self.set_pc(self.pc() + inst.size as u16);
            }
        }
        let intr_service_routine_cycles = if self.ime { self.run_interrupts() } else { 0 };
        if intr_service_routine_cycles != 0 && self.halted {
            self.halted = false;
        }
        if !self.halted {
            if self.debugger.step {
                self.set_paused(true);
                self.debugger.step = false;
            }

            let inst_cycles = if inst.kind == InstructionKind::Jump && !self.mutated_pc() {
                inst.cycles_not_taken
            } else {
                inst.cycles
            };
            self.set_clock_cycles(
                self.clock_cycles() + inst_cycles as u64 + intr_service_routine_cycles as u64,
            );
            self.set_inst_cycle_count(0);
            self.set_mutated_pc(false);
        }
        1 + intr_service_routine_cycles
    }

    /// To run an interrupt, we first run the interrupt service routine (ISR).
    /// That on itself takes 5 M-cycles, or 20 T-cycles.
    fn run_interrupts(&mut self) -> u8 {
        let iflag = self.bus.iflag();
        let intr = iflag & self.bus.ie();
        let isv;
        let intr_bit = if bw::test_bit8::<0>(intr) {
            isv = 0x40;
            0
        } else if bw::test_bit8::<1>(intr) {
            isv = 0x48;
            1
        } else if bw::test_bit8::<2>(intr) {
            isv = 0x50;
            2
        } else if bw::test_bit8::<3>(intr) {
            isv = 0x58;
            3
        } else if bw::test_bit8::<4>(intr) {
            isv = 0x60;
            4
        } else {
            // No interrupt is pending
            return 0;
        };
        self.bus.set_iflag(bw::set_bit8_dyn(iflag, intr_bit, false));
        self.set_ime(false);
        self.push(self.pc());
        self.set_pc(isv);
        // https://gbdev.io/pandocs/Interrupts.html#interrupt-handling
        20
    }

    /// Run one complete instruction - NOT a machine cycle (4 t-cycles)
    pub fn instruction(&mut self) -> u8 {
        let instruction = self.decode();
        let mut cycles_ran = 0;
        while cycles_ran < instruction.cycles {
            cycles_ran += self.step();
        }
        cycles_ran
    }

    fn execute(&mut self, instruction: Instruction) {
        if self.ime_next_inst {
            self.set_ime(true);
            self.ime_next_inst = false;
        }
        if self.prefix_cb {
            self.prefix_cb = false;
        }

        match instruction.opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC,d16
                self.set_bc(self.get_d16(0));
            }
            0x02 => {
                // LD (BC),A
                self.set_mem8(self.bc(), self.a());
            }
            0x03 => {
                // INC BC
                let result = self.inc16(self.bc());
                self.set_bc(result);
            }
            0x04 => {
                // INC B
                let result = self.inc8(self.b());
                self.set_b(result);
            }
            0x05 => {
                // DEC B
                let result = self.dec8(self.b());
                self.set_b(result);
            }
            0x06 => {
                // LD B,d8
                self.set_b(self.get_d8(0));
            }
            0x07 => {
                // RLCA
                let result = self.rlc8(self.a());
                self.set_z_flag(false);
                self.set_a(result);
            }
            0x08 => {
                // LD (a16),SP
                self.set_mem16(self.get_d16(0), self.sp());
            }
            0x09 => {
                // ADD HL,BC
                let result = self.add16(self.hl(), self.bc());
                self.set_hl(result);
            }
            0x0A => {
                // LD A,(BC)
                self.set_a(self.mem8(self.bc()));
            }
            0x0B => {
                // DEC BC
                let result = self.dec16(self.bc());
                self.set_bc(result);
            }
            0x0C => {
                // INC C
                let result = self.inc8(self.c());
                self.set_c(result);
            }
            0x0D => {
                // DEC C
                let result = self.dec8(self.c());
                self.set_c(result);
            }
            0x0E => {
                // LD C,d8
                self.set_c(self.get_d8(0));
            }
            0x0F => {
                // RRCA
                let result = self.rrc8(self.a());
                self.set_z_flag(false);
                self.set_a(result);
            }
            0x10 => {
                // STOP 0
                todo!("0x10 STOP 0")
            }
            0x11 => {
                // LD DE,d16
                self.set_de(self.get_d16(0));
            }
            0x12 => {
                // LD (DE),A
                self.set_mem8(self.de(), self.a());
            }
            0x13 => {
                // INC DE
                let result = self.inc16(self.de());
                self.set_de(result);
            }
            0x14 => {
                // INC D
                let result: u8 = self.inc8(self.d());
                self.set_d(result);
            }
            0x15 => {
                // DEC D
                let result = self.dec8(self.d());
                self.set_d(result);
            }
            0x16 => {
                // LD D,d8
                self.set_d(self.get_d8(0));
            }
            0x17 => {
                // RLA
                let result = self.rl8(self.a());
                self.set_z_flag(false);
                self.set_a(result);
            }
            0x18 => {
                // JR r8
                self.jump(self.calc_jr_address(self.pc(), self.get_r8(0)));
            }
            0x19 => {
                // ADD HL,DE
                let result = self.add16(self.hl(), self.de());
                self.set_hl(result);
            }
            0x1A => {
                // LD A,(DE)
                self.set_a(self.mem8(self.de()));
            }
            0x1B => {
                // DEC DE
                let result = self.dec16(self.de());
                self.set_de(result);
            }
            0x1C => {
                // INC E
                let result = self.inc8(self.e());
                self.set_e(result);
            }
            0x1D => {
                // DEC E
                let result = self.dec8(self.e());
                self.set_e(result);
            }
            0x1E => {
                // LD E,d8
                self.set_e(self.get_d8(0));
            }
            0x1F => {
                // RRA
                let result = self.rr8(self.a());
                self.set_z_flag(false);
                self.set_a(result);
            }
            0x20 => {
                // JR NZ,r8
                if !self.z_flag() {
                    self.jump(self.calc_jr_address(self.pc(), self.get_r8(0)));
                }
            }
            0x21 => {
                // LD HL,d16
                self.set_hl(self.get_d16(0));
            }
            0x22 => {
                // LD (HL+),A
                self.set_mem8(self.hl(), self.a());
                self.set_hl(self.hl().overflowing_add(1).0);
            }
            0x23 => {
                // INC HL
                let result = self.inc16(self.hl());
                self.set_hl(result);
            }
            0x24 => {
                // INC H
                let result = self.inc8(self.h());
                self.set_h(result);
            }
            0x25 => {
                // DEC H
                let result = self.dec8(self.h());
                self.set_h(result);
            }
            0x26 => {
                // LD H,d8
                self.set_h(self.get_d8(0));
            }
            0x27 => {
                // DAA
                if !self.n_flag() {
                    if self.c_flag() || self.a() > 0x99 {
                        self.set_a(self.a() + 0x60);
                        self.set_c_flag(true);
                    }
                    if self.h_flag() || (self.a() & 0x0f) > 0x09 {
                        self.set_a(self.a() + 0x6);
                    }
                } else {
                    if self.c_flag() {
                        self.set_a(self.a() - 0x60);
                    }
                    if self.h_flag() {
                        self.set_a(self.a() - 0x6);
                    }
                }

                self.set_z_flag(self.a() == 0);
                self.set_h_flag(false);
            }
            0x28 => {
                // JR Z,r8
                if self.z_flag() {
                    self.jump(self.calc_jr_address(self.pc(), self.get_r8(0)));
                }
            }
            0x29 => {
                // ADD HL,HL
                let result = self.add16(self.hl(), self.hl());
                self.set_hl(result);
            }
            0x2A => {
                // LD A,(HL+)
                self.set_a(self.hl_ind());
                self.set_hl(self.hl().overflowing_add(1).0);
            }
            0x2B => {
                // DEC HL
                let result = self.dec16(self.hl());
                self.set_hl(result);
            }
            0x2C => {
                // INC L
                let result = self.inc8(self.l());
                self.set_l(result);
            }
            0x2D => {
                // DEC L
                let result = self.dec8(self.l());
                self.set_l(result);
            }
            0x2E => {
                // LD L,d8
                self.set_l(self.get_d8(0));
            }
            0x2F => {
                // CPL
                self.set_a(!self.a());
                self.set_n_flag(true);
                self.set_h_flag(true);
            }
            0x30 => {
                // JR NC,r8
                if !self.c_flag() {
                    self.jump(self.calc_jr_address(self.pc(), self.get_r8(0)));
                }
            }
            0x31 => {
                // LD SP,d16
                self.set_sp(self.get_d16(0));
            }
            0x32 => {
                // LD (HL-),A
                self.set_mem8(self.hl(), self.a());
                self.set_hl(self.hl().overflowing_sub(1).0)
            }
            0x33 => {
                // INC SP
                let result = self.inc16(self.sp());
                self.set_sp(result);
            }
            0x34 => {
                // INC (HL)
                let result = self.inc8(self.mem8(self.hl()));
                self.set_mem8(self.hl(), result);
            }
            0x35 => {
                // DEC (HL)
                let result = self.dec8(self.mem8(self.hl()));
                self.set_mem8(self.hl(), result);
            }
            0x36 => {
                // LD (HL),d8
                self.set_mem8(self.hl(), self.get_d8(0));
            }
            0x37 => {
                // SCF
                self.set_n_flag(false);
                self.set_h_flag(false);
                self.set_c_flag(true);
            }
            0x38 => {
                // JR C,r8
                if self.c_flag() {
                    self.jump(self.calc_jr_address(self.pc(), self.get_r8(0)));
                }
            }
            0x39 => {
                // ADD HL,SP
                let result = self.add16(self.hl(), self.sp());
                self.set_hl(result);
            }
            0x3A => {
                // LD A,(HL-)
                self.set_a(self.hl_ind());
                self.set_hl(self.hl().overflowing_sub(1).0);
            }
            0x3B => {
                // DEC SP
                let result = self.dec16(self.sp());
                self.set_sp(result);
            }
            0x3C => {
                // INC A
                let result = self.inc8(self.a());
                self.set_a(result);
            }
            0x3D => {
                // DEC A
                let result = self.dec8(self.a());
                self.set_a(result);
            }
            0x3E => {
                // LD A,d8
                self.set_a(self.get_d8(0));
            }
            0x3F => {
                // CCF
                self.set_n_flag(false);
                self.set_h_flag(false);
                self.set_c_flag(!self.c_flag());
            }
            0x40 => {
                // LD B,B
                self.set_b(self.b());
            }
            0x41 => {
                // LD B,C
                self.set_b(self.c());
            }
            0x42 => {
                // LD B,D
                self.set_b(self.d());
            }
            0x43 => {
                // LD B,E
                self.set_b(self.e());
            }
            0x44 => {
                // LD B,H
                self.set_b(self.h());
            }
            0x45 => {
                // LD B,L
                self.set_b(self.l());
            }
            0x46 => {
                // LD B,(HL)
                self.set_b(self.hl_ind());
            }
            0x47 => {
                // LD B,A
                self.set_b(self.a());
            }
            0x48 => {
                // LD C,B
                self.set_c(self.b());
            }
            0x49 => {
                // LD C,C
                self.set_c(self.c());
            }
            0x4A => {
                // LD C,D
                self.set_c(self.d());
            }
            0x4B => {
                // LD C,E
                self.set_c(self.e());
            }
            0x4C => {
                // LD C,H
                self.set_c(self.h());
            }
            0x4D => {
                // LD C,L
                self.set_c(self.l());
            }
            0x4E => {
                // LD C,(HL)
                self.set_c(self.hl_ind());
            }
            0x4F => {
                // LD C,A
                self.set_c(self.a());
            }
            0x50 => {
                // LD D,B
                self.set_d(self.b());
            }
            0x51 => {
                // LD D,C
                self.set_d(self.c());
            }
            0x52 => {
                // LD D,D
                self.set_d(self.d());
            }
            0x53 => {
                // LD D,E
                self.set_d(self.e());
            }
            0x54 => {
                // LD D,H
                self.set_d(self.h());
            }
            0x55 => {
                // LD D,L
                self.set_d(self.l());
            }
            0x56 => {
                // LD D,(HL)
                self.set_d(self.hl_ind());
            }
            0x57 => {
                // LD D,A
                self.set_d(self.a());
            }
            0x58 => {
                // LD E,B
                self.set_e(self.b());
            }
            0x59 => {
                // LD E,C
                self.set_e(self.c());
            }
            0x5A => {
                // LD E,D
                self.set_e(self.d());
            }
            0x5B => {
                // LD E,E
                self.set_e(self.e());
            }
            0x5C => {
                // LD E,H
                self.set_e(self.h());
            }
            0x5D => {
                // LD E,L
                self.set_e(self.l());
            }
            0x5E => {
                // LD E,(HL)
                self.set_e(self.hl_ind());
            }
            0x5F => {
                // LD E,A
                self.set_e(self.a());
            }
            0x60 => {
                // LD H,B
                self.set_h(self.b());
            }
            0x61 => {
                // LD H,C
                self.set_h(self.c());
            }
            0x62 => {
                // LD H,D
                self.set_h(self.d());
            }
            0x63 => {
                // LD H,E
                self.set_h(self.e());
            }
            0x64 => {
                // LD H,H
                self.set_h(self.h());
            }
            0x65 => {
                // LD H,L
                self.set_h(self.l());
            }
            0x66 => {
                // LD H,(HL)
                self.set_h(self.hl_ind());
            }
            0x67 => {
                // LD H,A
                self.set_h(self.a());
            }
            0x68 => {
                // LD L,B
                self.set_l(self.b());
            }
            0x69 => {
                // LD L,C
                self.set_l(self.c());
            }
            0x6A => {
                // LD L,D
                self.set_l(self.d());
            }
            0x6B => {
                // LD L,E
                self.set_l(self.e());
            }
            0x6C => {
                // LD L,H
                self.set_l(self.h());
            }
            0x6D => {
                // LD L,L
                self.set_l(self.l());
            }
            0x6E => {
                // LD L,(HL)
                self.set_l(self.hl_ind());
            }
            0x6F => {
                // LD L,A
                self.set_l(self.a());
            }
            0x70 => {
                // LD (HL),B
                self.set_mem8(self.hl(), self.b());
            }
            0x71 => {
                // LD (HL),C
                self.set_mem8(self.hl(), self.c());
            }
            0x72 => {
                // LD (HL),D
                self.set_mem8(self.hl(), self.d());
            }
            0x73 => {
                // LD (HL),E
                self.set_mem8(self.hl(), self.e());
            }
            0x74 => {
                // LD (HL),H
                self.set_mem8(self.hl(), self.h());
            }
            0x75 => {
                // LD (HL),L
                self.set_mem8(self.hl(), self.l());
            }
            0x76 => {
                // HALT
                // TODO: Take care for halt bug: https://gbdev.io/pandocs/halt.html
                // https://rgbds.gbdev.io/docs/v0.6.1/gbz80.7/#HALT
                self.halted = true;
            }
            0x77 => {
                // LD (HL),A
                self.set_mem8(self.hl(), self.a());
            }
            0x78 => {
                // LD A,B
                self.set_a(self.b());
            }
            0x79 => {
                // LD A,C
                self.set_a(self.c());
            }
            0x7A => {
                // LD A,D
                self.set_a(self.d());
            }
            0x7B => {
                // LD A,E
                self.set_a(self.e());
            }
            0x7C => {
                // LD A,H
                self.set_a(self.h());
            }
            0x7D => {
                // LD A,L
                self.set_a(self.l());
            }
            0x7E => {
                // LD A,(HL)
                self.set_a(self.hl_ind());
            }
            0x7F => {
                // LD A,A
                self.set_a(self.a());
            }
            0x80 => {
                // ADD A,B
                let result = self.add8(self.a(), self.b());
                self.set_a(result);
            }
            0x81 => {
                // ADD A,C
                let result = self.add8(self.a(), self.c());
                self.set_a(result);
            }
            0x82 => {
                // ADD A,D
                let result = self.add8(self.a(), self.d());
                self.set_a(result);
            }
            0x83 => {
                // ADD A,E
                let result = self.add8(self.a(), self.e());
                self.set_a(result);
            }
            0x84 => {
                // ADD A,H
                let result = self.add8(self.a(), self.h());
                self.set_a(result);
            }
            0x85 => {
                // ADD A,L
                let result = self.add8(self.a(), self.l());
                self.set_a(result);
            }
            0x86 => {
                // ADD A,(HL)
                let result = self.add8(self.a(), self.hl_ind());
                self.set_a(result);
            }
            0x87 => {
                // ADD A,A
                let result = self.add8(self.a(), self.a());
                self.set_a(result);
            }
            0x88 => {
                // ADC A,B
                let result = self.addc8(self.a(), self.b());
                self.set_a(result);
            }
            0x89 => {
                // ADC A,C
                let result = self.addc8(self.a(), self.c());
                self.set_a(result);
            }
            0x8A => {
                // ADC A,D
                let result = self.addc8(self.a(), self.d());
                self.set_a(result);
            }
            0x8B => {
                // ADC A,E
                let result = self.addc8(self.a(), self.e());
                self.set_a(result);
            }
            0x8C => {
                // ADC A,H
                let result = self.addc8(self.a(), self.h());
                self.set_a(result);
            }
            0x8D => {
                // ADC A,L
                let result = self.addc8(self.a(), self.l());
                self.set_a(result);
            }
            0x8E => {
                // ADC A,(HL)
                let result = self.addc8(self.a(), self.hl_ind());
                self.set_a(result);
            }
            0x8F => {
                // ADC A,A
                let result = self.addc8(self.a(), self.a());
                self.set_a(result);
            }
            0x90 => {
                // SUB B
                let result = self.sub8(self.a(), self.b());
                self.set_a(result);
            }
            0x91 => {
                // SUB C
                let result = self.sub8(self.a(), self.c());
                self.set_a(result);
            }
            0x92 => {
                // SUB D
                let result = self.sub8(self.a(), self.d());
                self.set_a(result);
            }
            0x93 => {
                // SUB E
                let result = self.sub8(self.a(), self.e());
                self.set_a(result);
            }
            0x94 => {
                // SUB H
                let result = self.sub8(self.a(), self.h());
                self.set_a(result);
            }
            0x95 => {
                // SUB L
                let result = self.sub8(self.a(), self.l());
                self.set_a(result);
            }
            0x96 => {
                // SUB (HL)
                let result = self.sub8(self.a(), self.hl_ind());
                self.set_a(result);
            }
            0x97 => {
                // SUB A
                let result = self.sub8(self.a(), self.a());
                self.set_a(result);
            }
            0x98 => {
                // SBC A,B
                let result = self.subc8(self.a(), self.b());
                self.set_a(result);
            }
            0x99 => {
                // SBC A,C
                let result = self.subc8(self.a(), self.c());
                self.set_a(result);
            }
            0x9A => {
                // SBC A,D
                let result = self.subc8(self.a(), self.d());
                self.set_a(result);
            }
            0x9B => {
                // SBC A,E
                let result = self.subc8(self.a(), self.e());
                self.set_a(result);
            }
            0x9C => {
                // SBC A,H
                let result = self.subc8(self.a(), self.h());
                self.set_a(result);
            }
            0x9D => {
                // SBC A,L
                let result = self.subc8(self.a(), self.l());
                self.set_a(result);
            }
            0x9E => {
                // SBC A,(HL)
                let result = self.subc8(self.a(), self.hl_ind());
                self.set_a(result);
            }
            0x9F => {
                // SBC A,A
                let result = self.subc8(self.a(), self.a());
                self.set_a(result);
            }
            0xA0 => {
                // AND B
                let result = self.and8(self.b());
                self.set_a(result);
            }
            0xA1 => {
                // AND C
                let result = self.and8(self.c());
                self.set_a(result);
            }
            0xA2 => {
                // AND D
                let result = self.and8(self.d());
                self.set_a(result);
            }
            0xA3 => {
                // AND E
                let result = self.and8(self.e());
                self.set_a(result);
            }
            0xA4 => {
                // AND H
                let result = self.and8(self.h());
                self.set_a(result);
            }
            0xA5 => {
                // AND L
                let result = self.and8(self.l());
                self.set_a(result);
            }
            0xA6 => {
                // AND (HL)
                let result = self.and8(self.hl_ind());
                self.set_a(result);
            }
            0xA7 => {
                // AND A
                let result = self.and8(self.a());
                self.set_a(result);
            }
            0xA8 => {
                // XOR B
                let result = self.xor8(self.b());
                self.set_a(result);
            }
            0xA9 => {
                // XOR C
                let result = self.xor8(self.c());
                self.set_a(result);
            }
            0xAA => {
                // XOR D
                let result = self.xor8(self.d());
                self.set_a(result);
            }
            0xAB => {
                // XOR E
                let result = self.xor8(self.e());
                self.set_a(result);
            }
            0xAC => {
                // XOR H
                let result = self.xor8(self.h());
                self.set_a(result);
            }
            0xAD => {
                // XOR L
                let result = self.xor8(self.l());
                self.set_a(result);
            }
            0xAE => {
                // XOR (HL)
                let result = self.xor8(self.hl_ind());
                self.set_a(result);
            }
            0xAF => {
                // XOR A
                let result = self.xor8(self.a());
                self.set_a(result);
            }
            0xB0 => {
                // OR B
                let result = self.or8(self.b());
                self.set_a(result);
            }
            0xB1 => {
                // OR C
                let result = self.or8(self.c());
                self.set_a(result);
            }
            0xB2 => {
                // OR D
                let result = self.or8(self.d());
                self.set_a(result);
            }
            0xB3 => {
                // OR E
                let result = self.or8(self.e());
                self.set_a(result);
            }
            0xB4 => {
                // OR H
                let result = self.or8(self.h());
                self.set_a(result);
            }
            0xB5 => {
                // OR L
                let result = self.or8(self.l());
                self.set_a(result);
            }
            0xB6 => {
                // OR (HL)
                let result = self.or8(self.hl_ind());
                self.set_a(result);
            }
            0xB7 => {
                // OR A
                let result = self.or8(self.a());
                self.set_a(result);
            }
            0xB8 => {
                // CP B
                self.sub8(self.a(), self.b());
            }
            0xB9 => {
                // CP C
                self.sub8(self.a(), self.c());
            }
            0xBA => {
                // CP D
                self.sub8(self.a(), self.d());
            }
            0xBB => {
                // CP E
                self.sub8(self.a(), self.e());
            }
            0xBC => {
                // CP H
                self.sub8(self.a(), self.h());
            }
            0xBD => {
                // CP L
                self.sub8(self.a(), self.l());
            }
            0xBE => {
                // CP (HL)
                self.sub8(self.a(), self.hl_ind());
            }
            0xBF => {
                // CP A
                self.sub8(self.a(), self.a());
            }
            0xC0 => {
                // RET NZ
                if !self.z_flag() {
                    self.ret()
                }
            }
            0xC1 => {
                // POP BC
                let value = self.pop();
                self.set_bc(value);
            }
            0xC2 => {
                // JP NZ,a16
                if !self.z_flag() {
                    self.jump(self.get_d16(0));
                }
            }
            0xC3 => {
                // JP a16
                self.jump(self.get_d16(0));
            }
            0xC4 => {
                // CALL NZ,a16
                if !self.z_flag() {
                    self.call(self.get_d16(0));
                }
            }
            0xC5 => {
                // PUSH BC
                self.push(self.bc());
            }
            0xC6 => {
                // ADD A,d8
                let result = self.add8(self.a(), self.get_d8(0));
                self.set_a(result);
            }
            0xC7 => {
                // RST 00H
                self.rst(0x00);
            }
            0xC8 => {
                // RET Z
                if self.z_flag() {
                    self.ret()
                }
            }
            0xC9 => {
                // RET
                self.ret();
            }
            0xCA => {
                // JP Z,a16
                if self.z_flag() {
                    self.jump(self.get_d16(0));
                }
            }
            0xCB => {
                // PREFIX CB
                self.prefix_cb = true;
            }
            0xCC => {
                // CALL Z,a16
                if self.z_flag() {
                    self.call(self.get_d16(0));
                }
            }
            0xCD => {
                // CALL a16
                self.call(self.get_d16(0));
            }
            0xCE => {
                // ADC A,d8
                let result = self.addc8(self.a(), self.get_d8(0));
                self.set_a(result);
            }
            0xCF => {
                // RST 08H
                self.rst(0x08);
            }
            0xD0 => {
                // RET NC
                if !self.c_flag() {
                    self.ret();
                }
            }
            0xD1 => {
                // POP DE
                let value = self.pop();
                self.set_de(value);
            }
            0xD2 => {
                // JP NC,a16
                if !self.c_flag() {
                    self.jump(self.get_d16(0));
                }
            }
            0xD3 => {
                // Not implemented
                unimplemented!()
            }
            0xD4 => {
                // CALL NC,a16
                if !self.c_flag() {
                    self.call(self.get_d16(0));
                }
            }
            0xD5 => {
                // PUSH DE
                self.push(self.de());
            }
            0xD6 => {
                // SUB d8
                let result = self.sub8(self.a(), self.get_d8(0));
                self.set_a(result);
            }
            0xD7 => {
                // RST 10H
                self.rst(0x10);
            }
            0xD8 => {
                // RET C
                if self.c_flag() {
                    self.ret();
                }
            }
            0xD9 => {
                // RETI
                self.reti();
            }
            0xDA => {
                // JP C,a16
                if self.c_flag() {
                    self.jump(self.get_d16(0));
                }
            }
            0xDB => {
                // Not implemented
                unimplemented!()
            }
            0xDC => {
                // CALL C,a16
                if self.c_flag() {
                    self.call(self.get_d16(0));
                }
            }
            0xDD => {
                // Not implemented
                unimplemented!()
            }
            0xDE => {
                // SBC A,d8
                let result = self.subc8(self.a(), self.get_d8(0));
                self.set_a(result);
            }
            0xDF => {
                // RST 18H
                self.rst(0x18);
            }
            0xE0 => {
                // LDH (a8),A
                self.set_mem8(0xFF00 | self.get_d8(0) as u16, self.a());
            }
            0xE1 => {
                // POP HL
                let value = self.pop();
                self.set_hl(value);
            }
            0xE2 => {
                // LD (C),A
                self.set_mem8(0xFF00 + self.c() as u16, self.a());
            }
            0xE3 => {
                // Not implemented
                unimplemented!()
            }
            0xE4 => {
                // Not implemented
                unimplemented!()
            }
            0xE5 => {
                // PUSH HL
                self.push(self.hl());
            }
            0xE6 => {
                // AND d8
                let result = self.and8(self.get_d8(0));
                self.set_a(result);
            }
            0xE7 => {
                // RST 20H
                self.rst(0x20);
            }
            0xE8 => {
                // ADD SP,r8
                let result = self.add16i(self.sp(), self.get_r8(0));
                self.set_sp(result);
            }
            0xE9 => {
                // JP (HL)
                self.jump(self.hl());
            }
            0xEA => {
                // LD (a16),A
                self.set_mem8(self.get_d16(0), self.a());
            }
            0xEB => {
                // Not implemented
                unimplemented!()
            }
            0xEC => {
                // Not implemented
                unimplemented!()
            }
            0xED => {
                // Not implemented
                unimplemented!()
            }
            0xEE => {
                // XOR d8
                let result = self.xor8(self.get_d8(0));
                self.set_a(result);
            }
            0xEF => {
                // RST 28H
                self.rst(0x28);
            }
            0xF0 => {
                // LDH A,(a8)
                self.set_a(self.mem8(0xFF00 | self.get_d8(0) as u16));
            }
            0xF1 => {
                // POP AF
                let value = self.pop();
                self.set_af(value);
            }
            0xF2 => {
                // LD A,(C)
                self.set_a(self.mem8(0xFF00 | self.c() as u16));
            }
            0xF3 => {
                // DI
                self.set_ime(false);
            }
            0xF4 => {
                // Not implemented
                unimplemented!()
            }
            0xF5 => {
                // PUSH AF
                self.push(self.af());
            }
            0xF6 => {
                // OR d8
                let result = self.or8(self.get_d8(0));
                self.set_a(result);
            }
            0xF7 => {
                // RST 30H
                self.rst(0x30);
            }
            0xF8 => {
                // LD HL,SP+r8
                let result = self.add16i(self.sp(), self.get_r8(0));
                self.set_hl(result);
            }
            0xF9 => {
                // LD SP,HL
                self.set_sp(self.hl());
            }
            0xFA => {
                // LD A,(a16)
                self.set_a(self.mem8(self.get_d16(0)));
            }
            0xFB => {
                // EI
                self.set_ime_next_inst();
            }
            0xFC => {
                // Not implemented
                unimplemented!()
            }
            0xFD => {
                // Not implemented
                unimplemented!()
            }
            0xFE => {
                // CP d8
                self.sub8(self.a(), self.get_d8(0));
            }
            0xFF => {
                // RST 38H
                self.rst(0x38);
            }
            0x100 => {
                // RLC B
                let result = self.rlc8(self.b());
                self.set_b(result);
            }
            0x101 => {
                // RLC C
                let result = self.rlc8(self.c());
                self.set_c(result);
            }
            0x102 => {
                // RLC D
                let result = self.rlc8(self.d());
                self.set_d(result);
            }
            0x103 => {
                // RLC E
                let result = self.rlc8(self.e());
                self.set_e(result);
            }
            0x104 => {
                // RLC H
                let result = self.rlc8(self.h());
                self.set_h(result);
            }
            0x105 => {
                // RLC L
                let result = self.rlc8(self.l());
                self.set_l(result);
            }
            0x106 => {
                // RLC (HL)
                let result = self.rlc8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x107 => {
                // RLC A
                let result = self.rlc8(self.a());
                self.set_a(result);
            }
            0x108 => {
                // RRC B
                let result = self.rrc8(self.b());
                self.set_b(result);
            }
            0x109 => {
                // RRC C
                let result = self.rrc8(self.c());
                self.set_c(result);
            }
            0x10A => {
                // RRC D
                let result = self.rrc8(self.d());
                self.set_d(result);
            }
            0x10B => {
                // RRC E
                let result = self.rrc8(self.e());
                self.set_e(result);
            }
            0x10C => {
                // RRC H
                let result = self.rrc8(self.h());
                self.set_h(result);
            }
            0x10D => {
                // RRC L
                let result = self.rrc8(self.l());
                self.set_l(result);
            }
            0x10E => {
                // RRC (HL)
                let result = self.rrc8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x10F => {
                // RRC A
                let result = self.rrc8(self.a());
                self.set_a(result);
            }
            0x110 => {
                // RL B
                let result = self.rl8(self.b());
                self.set_b(result);
            }
            0x111 => {
                // RL C
                let result = self.rl8(self.c());
                self.set_c(result);
            }
            0x112 => {
                // RL D
                let result = self.rl8(self.d());
                self.set_d(result);
            }
            0x113 => {
                // RL E
                let result = self.rl8(self.e());
                self.set_e(result);
            }
            0x114 => {
                // RL H
                let result = self.rl8(self.h());
                self.set_h(result);
            }
            0x115 => {
                // RL L
                let result = self.rl8(self.l());
                self.set_l(result);
            }
            0x116 => {
                // RL (HL)
                let result = self.rl8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x117 => {
                // RL A
                let result = self.rl8(self.a());
                self.set_a(result);
            }
            0x118 => {
                // RR B
                let result = self.rr8(self.b());
                self.set_b(result);
            }
            0x119 => {
                // RR C
                let result = self.rr8(self.c());
                self.set_c(result);
            }
            0x11A => {
                // RR D
                let result = self.rr8(self.d());
                self.set_d(result);
            }
            0x11B => {
                // RR E
                let result = self.rr8(self.e());
                self.set_e(result);
            }
            0x11C => {
                // RR H
                let result = self.rr8(self.h());
                self.set_h(result);
            }
            0x11D => {
                // RR L
                let result = self.rr8(self.l());
                self.set_l(result);
            }
            0x11E => {
                // RR (HL)
                let result = self.rr8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x11F => {
                // RR A
                let result = self.rr8(self.a());
                self.set_a(result);
            }
            0x120 => {
                // SLA B
                let result = self.sla8(self.b());
                self.set_b(result);
            }
            0x121 => {
                // SLA C
                let result = self.sla8(self.c());
                self.set_c(result);
            }
            0x122 => {
                // SLA D
                let result = self.sla8(self.d());
                self.set_d(result);
            }
            0x123 => {
                // SLA E
                let result = self.sla8(self.e());
                self.set_e(result);
            }
            0x124 => {
                // SLA H
                let result = self.sla8(self.h());
                self.set_h(result);
            }
            0x125 => {
                // SLA L
                let result = self.sla8(self.l());
                self.set_l(result);
            }
            0x126 => {
                // SLA (HL)
                let result = self.sla8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x127 => {
                // SLA A
                let result = self.sla8(self.a());
                self.set_a(result);
            }
            0x128 => {
                // SRA B
                let result = self.sra8(self.b());
                self.set_b(result);
            }
            0x129 => {
                // SRA C
                let result = self.sra8(self.c());
                self.set_c(result);
            }
            0x12A => {
                // SRA D
                let result = self.sra8(self.d());
                self.set_d(result);
            }
            0x12B => {
                // SRA E
                let result = self.sra8(self.e());
                self.set_e(result);
            }
            0x12C => {
                // SRA H
                let result = self.sra8(self.h());
                self.set_h(result);
            }
            0x12D => {
                // SRA L
                let result = self.sra8(self.l());
                self.set_l(result);
            }
            0x12E => {
                // SRA (HL)
                let result = self.sra8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x12F => {
                // SRA A
                let result = self.sra8(self.a());
                self.set_a(result);
            }
            0x130 => {
                // SWAP B
                let result = self.swap8(self.b());
                self.set_b(result);
            }
            0x131 => {
                // SWAP C
                let result = self.swap8(self.c());
                self.set_c(result);
            }
            0x132 => {
                // SWAP D
                let result = self.swap8(self.d());
                self.set_d(result);
            }
            0x133 => {
                // SWAP E
                let result = self.swap8(self.e());
                self.set_e(result);
            }
            0x134 => {
                // SWAP H
                let result = self.swap8(self.h());
                self.set_h(result);
            }
            0x135 => {
                // SWAP L
                let result = self.swap8(self.l());
                self.set_l(result);
            }
            0x136 => {
                // SWAP (HL)
                let result = self.swap8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x137 => {
                // SWAP A
                let result = self.swap8(self.a());
                self.set_a(result);
            }
            0x138 => {
                // SRL B
                let result = self.srl8(self.b());
                self.set_b(result);
            }
            0x139 => {
                // SRL C
                let result = self.srl8(self.c());
                self.set_c(result);
            }
            0x13A => {
                // SRL D
                let result = self.srl8(self.d());
                self.set_d(result);
            }
            0x13B => {
                // SRL E
                let result = self.srl8(self.e());
                self.set_e(result);
            }
            0x13C => {
                // SRL H
                let result = self.srl8(self.h());
                self.set_h(result);
            }
            0x13D => {
                // SRL L
                let result = self.srl8(self.l());
                self.set_l(result);
            }
            0x13E => {
                // SRL (HL)
                let result = self.srl8(self.hl_ind());
                self.set_hl_ind(result);
            }
            0x13F => {
                // SRL A
                let result = self.srl8(self.a());
                self.set_a(result);
            }
            0x140 => {
                // BIT 0,B
                self.bit::<0>(self.b());
            }
            0x141 => {
                // BIT 0,C
                self.bit::<0>(self.c());
            }
            0x142 => {
                // BIT 0,D
                self.bit::<0>(self.d());
            }
            0x143 => {
                // BIT 0,E
                self.bit::<0>(self.e());
            }
            0x144 => {
                // BIT 0,H
                self.bit::<0>(self.h());
            }
            0x145 => {
                // BIT 0,L
                self.bit::<0>(self.l());
            }
            0x146 => {
                // BIT 0,(HL)
                self.bit::<0>(self.hl_ind());
            }
            0x147 => {
                // BIT 0,A
                self.bit::<0>(self.a());
            }
            0x148 => {
                // BIT 1,B
                self.bit::<1>(self.b());
            }
            0x149 => {
                // BIT 1,C
                self.bit::<1>(self.c());
            }
            0x14A => {
                // BIT 1,D
                self.bit::<1>(self.d());
            }
            0x14B => {
                // BIT 1,E
                self.bit::<1>(self.e());
            }
            0x14C => {
                // BIT 1,H
                self.bit::<1>(self.h());
            }
            0x14D => {
                // BIT 1,L
                self.bit::<1>(self.l());
            }
            0x14E => {
                // BIT 1,(HL)
                self.bit::<1>(self.hl_ind());
            }
            0x14F => {
                // BIT 1,A
                self.bit::<1>(self.a());
            }
            0x150 => {
                // BIT 2,B
                self.bit::<2>(self.b());
            }
            0x151 => {
                // BIT 2,C
                self.bit::<2>(self.c());
            }
            0x152 => {
                // BIT 2,D
                self.bit::<2>(self.d());
            }
            0x153 => {
                // BIT 2,E
                self.bit::<2>(self.e());
            }
            0x154 => {
                // BIT 2,H
                self.bit::<2>(self.h());
            }
            0x155 => {
                // BIT 2,L
                self.bit::<2>(self.l());
            }
            0x156 => {
                // BIT 2,(HL)
                self.bit::<2>(self.hl_ind());
            }
            0x157 => {
                // BIT 2,A
                self.bit::<2>(self.a());
            }
            0x158 => {
                // BIT 3,B
                self.bit::<3>(self.b());
            }
            0x159 => {
                // BIT 3,C
                self.bit::<3>(self.c());
            }
            0x15A => {
                // BIT 3,D
                self.bit::<3>(self.d());
            }
            0x15B => {
                // BIT 3,E
                self.bit::<3>(self.e());
            }
            0x15C => {
                // BIT 3,H
                self.bit::<3>(self.h());
            }
            0x15D => {
                // BIT 3,L
                self.bit::<3>(self.l());
            }
            0x15E => {
                // BIT 3,(HL)
                self.bit::<3>(self.hl_ind());
            }
            0x15F => {
                // BIT 3,A
                self.bit::<3>(self.a());
            }
            0x160 => {
                // BIT 4,B
                self.bit::<4>(self.b());
            }
            0x161 => {
                // BIT 4,C
                self.bit::<4>(self.c());
            }
            0x162 => {
                // BIT 4,D
                self.bit::<4>(self.d());
            }
            0x163 => {
                // BIT 4,E
                self.bit::<4>(self.e());
            }
            0x164 => {
                // BIT 4,H
                self.bit::<4>(self.h());
            }
            0x165 => {
                // BIT 4,L
                self.bit::<4>(self.l());
            }
            0x166 => {
                // BIT 4,(HL)
                self.bit::<4>(self.hl_ind());
            }
            0x167 => {
                // BIT 4,A
                self.bit::<4>(self.a());
            }
            0x168 => {
                // BIT 5,B
                self.bit::<5>(self.b());
            }
            0x169 => {
                // BIT 5,C
                self.bit::<5>(self.c());
            }
            0x16A => {
                // BIT 5,D
                self.bit::<5>(self.d());
            }
            0x16B => {
                // BIT 5,E
                self.bit::<5>(self.e());
            }
            0x16C => {
                // BIT 5,H
                self.bit::<5>(self.h());
            }
            0x16D => {
                // BIT 5,L
                self.bit::<5>(self.l());
            }
            0x16E => {
                // BIT 5,(HL)
                self.bit::<5>(self.hl_ind());
            }
            0x16F => {
                // BIT 5,A
                self.bit::<5>(self.a());
            }
            0x170 => {
                // BIT 6,B
                self.bit::<6>(self.b());
            }
            0x171 => {
                // BIT 6,C
                self.bit::<6>(self.c());
            }
            0x172 => {
                // BIT 6,D
                self.bit::<6>(self.d());
            }
            0x173 => {
                // BIT 6,E
                self.bit::<6>(self.e());
            }
            0x174 => {
                // BIT 6,H
                self.bit::<6>(self.h());
            }
            0x175 => {
                // BIT 6,L
                self.bit::<6>(self.l());
            }
            0x176 => {
                // BIT 6,(HL)
                self.bit::<6>(self.hl_ind());
            }
            0x177 => {
                // BIT 6,A
                self.bit::<6>(self.a());
            }
            0x178 => {
                // BIT 7,B
                self.bit::<7>(self.b());
            }
            0x179 => {
                // BIT 7,C
                self.bit::<7>(self.c());
            }
            0x17A => {
                // BIT 7,D
                self.bit::<7>(self.d());
            }
            0x17B => {
                // BIT 7,E
                self.bit::<7>(self.e());
            }
            0x17C => {
                // BIT 7,H
                self.bit::<7>(self.h());
            }
            0x17D => {
                // BIT 7,L
                self.bit::<7>(self.l());
            }
            0x17E => {
                // BIT 7,(HL)
                self.bit::<7>(self.hl_ind());
            }
            0x17F => {
                // BIT 7,A
                self.bit::<7>(self.a());
            }
            0x180 => {
                // RES 0,B
                self.set_b(bw::set_bit8::<0>(self.b(), false));
            }
            0x181 => {
                // RES 0,C
                self.set_c(bw::set_bit8::<0>(self.c(), false));
            }
            0x182 => {
                // RES 0,D
                self.set_d(bw::set_bit8::<0>(self.d(), false));
            }
            0x183 => {
                // RES 0,E
                self.set_e(bw::set_bit8::<0>(self.e(), false));
            }
            0x184 => {
                // RES 0,H
                self.set_h(bw::set_bit8::<0>(self.h(), false));
            }
            0x185 => {
                // RES 0,L
                self.set_l(bw::set_bit8::<0>(self.l(), false));
            }
            0x186 => {
                // RES 0,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<0>(self.hl_ind(), false));
            }
            0x187 => {
                // RES 0,A
                self.set_a(bw::set_bit8::<0>(self.a(), false));
            }
            0x188 => {
                // RES 1,B
                self.set_b(bw::set_bit8::<1>(self.b(), false));
            }
            0x189 => {
                // RES 1,C
                self.set_c(bw::set_bit8::<1>(self.c(), false));
            }
            0x18A => {
                // RES 1,D
                self.set_d(bw::set_bit8::<1>(self.d(), false));
            }
            0x18B => {
                // RES 1,E
                self.set_e(bw::set_bit8::<1>(self.e(), false));
            }
            0x18C => {
                // RES 1,H
                self.set_h(bw::set_bit8::<1>(self.h(), false));
            }
            0x18D => {
                // RES 1,L
                self.set_l(bw::set_bit8::<1>(self.l(), false));
            }
            0x18E => {
                // RES 1,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<1>(self.hl_ind(), false));
            }
            0x18F => {
                // RES 1,A
                self.set_a(bw::set_bit8::<1>(self.a(), false));
            }
            0x190 => {
                // RES 2,B
                self.set_b(bw::set_bit8::<2>(self.b(), false));
            }
            0x191 => {
                // RES 2,C
                self.set_c(bw::set_bit8::<2>(self.c(), false));
            }
            0x192 => {
                // RES 2,D
                self.set_d(bw::set_bit8::<2>(self.d(), false));
            }
            0x193 => {
                // RES 2,E
                self.set_e(bw::set_bit8::<2>(self.e(), false));
            }
            0x194 => {
                // RES 2,H
                self.set_h(bw::set_bit8::<2>(self.h(), false));
            }
            0x195 => {
                // RES 2,L
                self.set_l(bw::set_bit8::<2>(self.l(), false));
            }
            0x196 => {
                // RES 2,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<2>(self.hl_ind(), false));
            }
            0x197 => {
                // RES 2,A
                self.set_a(bw::set_bit8::<2>(self.a(), false));
            }
            0x198 => {
                // RES 3,B
                self.set_b(bw::set_bit8::<3>(self.b(), false));
            }
            0x199 => {
                // RES 3,C
                self.set_c(bw::set_bit8::<3>(self.c(), false));
            }
            0x19A => {
                // RES 3,D
                self.set_d(bw::set_bit8::<3>(self.d(), false));
            }
            0x19B => {
                // RES 3,E
                self.set_e(bw::set_bit8::<3>(self.e(), false));
            }
            0x19C => {
                // RES 3,H
                self.set_h(bw::set_bit8::<3>(self.h(), false));
            }
            0x19D => {
                // RES 3,L
                self.set_l(bw::set_bit8::<3>(self.l(), false));
            }
            0x19E => {
                // RES 3,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<3>(self.hl_ind(), false));
            }
            0x19F => {
                // RES 3,A
                self.set_a(bw::set_bit8::<3>(self.a(), false));
            }
            0x1A0 => {
                // RES 4,B
                self.set_b(bw::set_bit8::<4>(self.b(), false));
            }
            0x1A1 => {
                // RES 4,C
                self.set_c(bw::set_bit8::<4>(self.c(), false));
            }
            0x1A2 => {
                // RES 4,D
                self.set_d(bw::set_bit8::<4>(self.d(), false));
            }
            0x1A3 => {
                // RES 4,E
                self.set_e(bw::set_bit8::<4>(self.e(), false));
            }
            0x1A4 => {
                // RES 4,H
                self.set_h(bw::set_bit8::<4>(self.h(), false));
            }
            0x1A5 => {
                // RES 4,L
                self.set_l(bw::set_bit8::<4>(self.l(), false));
            }
            0x1A6 => {
                // RES 4,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<4>(self.hl_ind(), false));
            }
            0x1A7 => {
                // RES 4,A
                self.set_a(bw::set_bit8::<4>(self.a(), false));
            }
            0x1A8 => {
                // RES 5,B
                self.set_b(bw::set_bit8::<5>(self.b(), false));
            }
            0x1A9 => {
                // RES 5,C
                self.set_c(bw::set_bit8::<5>(self.c(), false));
            }
            0x1AA => {
                // RES 5,D
                self.set_d(bw::set_bit8::<5>(self.d(), false));
            }
            0x1AB => {
                // RES 5,E
                self.set_e(bw::set_bit8::<5>(self.e(), false));
            }
            0x1AC => {
                // RES 5,H
                self.set_h(bw::set_bit8::<5>(self.h(), false));
            }
            0x1AD => {
                // RES 5,L
                self.set_l(bw::set_bit8::<5>(self.l(), false));
            }
            0x1AE => {
                // RES 5,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<5>(self.hl_ind(), false));
            }
            0x1AF => {
                // RES 5,A
                self.set_a(bw::set_bit8::<5>(self.a(), false));
            }
            0x1B0 => {
                // RES 6,B
                self.set_b(bw::set_bit8::<6>(self.b(), false));
            }
            0x1B1 => {
                // RES 6,C
                self.set_c(bw::set_bit8::<6>(self.c(), false));
            }
            0x1B2 => {
                // RES 6,D
                self.set_d(bw::set_bit8::<6>(self.d(), false));
            }
            0x1B3 => {
                // RES 6,E
                self.set_e(bw::set_bit8::<6>(self.e(), false));
            }
            0x1B4 => {
                // RES 6,H
                self.set_h(bw::set_bit8::<6>(self.h(), false));
            }
            0x1B5 => {
                // RES 6,L
                self.set_l(bw::set_bit8::<6>(self.l(), false));
            }
            0x1B6 => {
                // RES 6,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<6>(self.hl_ind(), false));
            }
            0x1B7 => {
                // RES 6,A
                self.set_a(bw::set_bit8::<6>(self.a(), false));
            }
            0x1B8 => {
                // RES 7,B
                self.set_b(bw::set_bit8::<7>(self.b(), false));
            }
            0x1B9 => {
                // RES 7,C
                self.set_c(bw::set_bit8::<7>(self.c(), false));
            }
            0x1BA => {
                // RES 7,D
                self.set_d(bw::set_bit8::<7>(self.d(), false));
            }
            0x1BB => {
                // RES 7,E
                self.set_e(bw::set_bit8::<7>(self.e(), false));
            }
            0x1BC => {
                // RES 7,H
                self.set_h(bw::set_bit8::<7>(self.h(), false));
            }
            0x1BD => {
                // RES 7,L
                self.set_l(bw::set_bit8::<7>(self.l(), false));
            }
            0x1BE => {
                // RES 7,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<7>(self.hl_ind(), false));
            }
            0x1BF => {
                // RES 7,A
                self.set_a(bw::set_bit8::<7>(self.a(), false));
            }
            0x1C0 => {
                // SET 0,B
                self.set_b(bw::set_bit8::<0>(self.b(), true));
            }
            0x1C1 => {
                // SET 0,C
                self.set_c(bw::set_bit8::<0>(self.c(), true));
            }
            0x1C2 => {
                // SET 0,D
                self.set_d(bw::set_bit8::<0>(self.d(), true));
            }
            0x1C3 => {
                // SET 0,E
                self.set_e(bw::set_bit8::<0>(self.e(), true));
            }
            0x1C4 => {
                // SET 0,H
                self.set_h(bw::set_bit8::<0>(self.h(), true));
            }
            0x1C5 => {
                // SET 0,L
                self.set_l(bw::set_bit8::<0>(self.l(), true));
            }
            0x1C6 => {
                // SET 0,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<0>(self.hl_ind(), true));
            }
            0x1C7 => {
                // SET 0,A
                self.set_a(bw::set_bit8::<0>(self.a(), true));
            }
            0x1C8 => {
                // SET 1,B
                self.set_b(bw::set_bit8::<1>(self.b(), true));
            }
            0x1C9 => {
                // SET 1,C
                self.set_c(bw::set_bit8::<1>(self.c(), true));
            }
            0x1CA => {
                // SET 1,D
                self.set_d(bw::set_bit8::<1>(self.d(), true));
            }
            0x1CB => {
                // SET 1,E
                self.set_e(bw::set_bit8::<1>(self.e(), true));
            }
            0x1CC => {
                // SET 1,H
                self.set_h(bw::set_bit8::<1>(self.h(), true));
            }
            0x1CD => {
                // SET 1,L
                self.set_l(bw::set_bit8::<1>(self.l(), true));
            }
            0x1CE => {
                // SET 1,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<1>(self.hl_ind(), true));
            }
            0x1CF => {
                // SET 1,A
                self.set_a(bw::set_bit8::<1>(self.a(), true));
            }
            0x1D0 => {
                // SET 2,B
                self.set_b(bw::set_bit8::<2>(self.b(), true));
            }
            0x1D1 => {
                // SET 2,C
                self.set_c(bw::set_bit8::<2>(self.c(), true));
            }
            0x1D2 => {
                // SET 2,D
                self.set_d(bw::set_bit8::<2>(self.d(), true));
            }
            0x1D3 => {
                // SET 2,E
                self.set_e(bw::set_bit8::<2>(self.e(), true));
            }
            0x1D4 => {
                // SET 2,H
                self.set_h(bw::set_bit8::<2>(self.h(), true));
            }
            0x1D5 => {
                // SET 2,L
                self.set_l(bw::set_bit8::<2>(self.l(), true));
            }
            0x1D6 => {
                // SET 2,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<2>(self.hl_ind(), true));
            }
            0x1D7 => {
                // SET 2,A
                self.set_a(bw::set_bit8::<2>(self.a(), true));
            }
            0x1D8 => {
                // SET 3,B
                self.set_b(bw::set_bit8::<3>(self.b(), true));
            }
            0x1D9 => {
                // SET 3,C
                self.set_c(bw::set_bit8::<3>(self.c(), true));
            }
            0x1DA => {
                // SET 3,D
                self.set_d(bw::set_bit8::<3>(self.d(), true));
            }
            0x1DB => {
                // SET 3,E
                self.set_e(bw::set_bit8::<3>(self.e(), true));
            }
            0x1DC => {
                // SET 3,H
                self.set_h(bw::set_bit8::<3>(self.h(), true));
            }
            0x1DD => {
                // SET 3,L
                self.set_l(bw::set_bit8::<3>(self.l(), true));
            }
            0x1DE => {
                // SET 3,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<3>(self.hl_ind(), true));
            }
            0x1DF => {
                // SET 3,A
                self.set_a(bw::set_bit8::<3>(self.a(), true));
            }
            0x1E0 => {
                // SET 4,B
                self.set_b(bw::set_bit8::<4>(self.b(), true));
            }
            0x1E1 => {
                // SET 4,C
                self.set_c(bw::set_bit8::<4>(self.c(), true));
            }
            0x1E2 => {
                // SET 4,D
                self.set_d(bw::set_bit8::<4>(self.d(), true));
            }
            0x1E3 => {
                // SET 4,E
                self.set_e(bw::set_bit8::<4>(self.e(), true));
            }
            0x1E4 => {
                // SET 4,H
                self.set_h(bw::set_bit8::<4>(self.h(), true));
            }
            0x1E5 => {
                // SET 4,L
                self.set_l(bw::set_bit8::<4>(self.l(), true));
            }
            0x1E6 => {
                // SET 4,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<4>(self.hl_ind(), true));
            }
            0x1E7 => {
                // SET 4,A
                self.set_a(bw::set_bit8::<4>(self.a(), true));
            }
            0x1E8 => {
                // SET 5,B
                self.set_b(bw::set_bit8::<5>(self.b(), true));
            }
            0x1E9 => {
                // SET 5,C
                self.set_c(bw::set_bit8::<5>(self.c(), true));
            }
            0x1EA => {
                // SET 5,D
                self.set_d(bw::set_bit8::<5>(self.d(), true));
            }
            0x1EB => {
                // SET 5,E
                self.set_e(bw::set_bit8::<5>(self.e(), true));
            }
            0x1EC => {
                // SET 5,H
                self.set_h(bw::set_bit8::<5>(self.h(), true));
            }
            0x1ED => {
                // SET 5,L
                self.set_l(bw::set_bit8::<5>(self.l(), true));
            }
            0x1EE => {
                // SET 5,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<5>(self.hl_ind(), true));
            }
            0x1EF => {
                // SET 5,A
                self.set_a(bw::set_bit8::<5>(self.a(), true));
            }
            0x1F0 => {
                // SET 6,B
                self.set_b(bw::set_bit8::<6>(self.b(), true));
            }
            0x1F1 => {
                // SET 6,C
                self.set_c(bw::set_bit8::<6>(self.c(), true));
            }
            0x1F2 => {
                // SET 6,D
                self.set_d(bw::set_bit8::<6>(self.d(), true));
            }
            0x1F3 => {
                // SET 6,E
                self.set_e(bw::set_bit8::<6>(self.e(), true));
            }
            0x1F4 => {
                // SET 6,H
                self.set_h(bw::set_bit8::<6>(self.h(), true));
            }
            0x1F5 => {
                // SET 6,L
                self.set_l(bw::set_bit8::<6>(self.l(), true));
            }
            0x1F6 => {
                // SET 6,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<6>(self.hl_ind(), true));
            }
            0x1F7 => {
                // SET 6,A
                self.set_a(bw::set_bit8::<6>(self.a(), true));
            }
            0x1F8 => {
                // SET 7,B
                self.set_b(bw::set_bit8::<7>(self.b(), true));
            }
            0x1F9 => {
                // SET 7,C
                self.set_c(bw::set_bit8::<7>(self.c(), true));
            }
            0x1FA => {
                // SET 7,D
                self.set_d(bw::set_bit8::<7>(self.d(), true));
            }
            0x1FB => {
                // SET 7,E
                self.set_e(bw::set_bit8::<7>(self.e(), true));
            }
            0x1FC => {
                // SET 7,H
                self.set_h(bw::set_bit8::<7>(self.h(), true));
            }
            0x1FD => {
                // SET 7,L
                self.set_l(bw::set_bit8::<7>(self.l(), true));
            }
            0x1FE => {
                // SET 7,(HL)
                self.set_mem8(self.hl(), bw::set_bit8::<7>(self.hl_ind(), true));
            }
            0x1FF => {
                // SET 7,A
                self.set_a(bw::set_bit8::<7>(self.a(), true));
            }
            i => {
                unimplemented!("no idea what opcode {i} is")
            }
        }
    }
}
