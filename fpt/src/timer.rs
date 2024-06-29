use super::memory::Bus;
use crate::bw;
use crate::memory::map;

pub struct Timer {
    sys: u16, // system timer counter
    interrupt: bool,
    bus: Bus,
    div: u8,
    tima: u8,
    tac: u8,
    tma: u8,
}

impl Timer {
    pub fn new(memory: Bus) -> Self {
        Self {
            sys: 0,
            interrupt: false,
            bus: memory,
            div: 0,
            tima: 0,
            tac: 0,
            tma: 0,
        }
    }

    fn set_div(&mut self, div: u8) {
        self.div = div;
        self.bus.write(map::DIV, div);
    }

    fn set_tima(&mut self, tima: u8) {
        self.tima = tima;
        self.bus.write(map::TIMA, tima);
    }

    fn set_tac(&mut self, tac: u8) {
        self.tac = tac;
        self.bus.write(map::TAC, tac);
    }

    fn set_tma(&mut self, tma: u8) {
        self.tma = tma;
        self.bus.write(map::TMA, tma);
    }

    fn get_div(&self) -> u8 {
        let div = self.bus.read(map::DIV);
        if div != self.div {
            0
        } else {
            div
        }
    }

    fn get_tima(&self) -> u8 {
        self.bus.read(map::TIMA)
    }

    fn get_tac(&self) -> u8 {
        self.bus.read(map::TAC)
    }

    fn get_tma(&self) -> u8 {
        self.bus.read(map::TMA)
    }

    fn get_interrupt(&self) -> bool {
        self.interrupt
    }

    // Call this every M-cycle
    pub fn step(&mut self) {
        let mut div = self.get_div();
        let mut tima = self.get_tima();
        let tac = self.get_tac();
        let tma = self.get_tma();

        if (div + 1) % 4 == 0 {
            println!(
                "self.div: {}, self.tima: {}, self.tac: {}, self.tma: {}",
                div, tima, tac, tma
            );
        }

        self.sys = self.sys.overflowing_add(1).0;

        let enable = bw::test_bit8::<2>(tac);

        if !enable {
            return;
        }

        div = div.overflowing_add(1).0;

        let clock_select = tac & 0b11;

        let clock_select: u16 = match clock_select {
            0b00 => 256,
            0b01 => 4,
            0b10 => 16,
            0b11 => 64,
            _ => panic!(),
        };

        if div != 0 && (div as u16) % clock_select == 0 {
            tima = tima.overflowing_add(1).0;
        }

        if tima == 0 && enable {
            tima = tma;
            self.interrupt = true;
        } else {
            self.interrupt = false;
        }

        self.set_div(div);
        self.set_tima(dbg!(tima));
        self.set_tac(tac);
        self.set_tma(tma);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_timer() {
        let mut bus = Bus::new();
        let mut timer = Timer::new(bus.clone());

        bus.write(map::TAC, 0b101);
        bus.write(map::TMA, 0xFE);

        timer.step();

        assert_eq!(bus.read(map::DIV), 1);
        assert_eq!(bus.read(map::TIMA), 254);

        timer.step();

        assert_eq!(bus.read(map::DIV), 2);
        assert_eq!(bus.read(map::TIMA), 254);

        timer.step();

        assert_eq!(bus.read(map::DIV), 3);
        assert_eq!(bus.read(map::TIMA), 254);

        timer.step();

        assert_eq!(bus.read(map::DIV), 4);
        assert_eq!(bus.read(map::TIMA), 255);

        timer.step();

        assert_eq!(bus.read(map::DIV), 5);
        assert_eq!(bus.read(map::TIMA), 255);

        timer.step();

        assert_eq!(bus.read(map::DIV), 6);
        assert_eq!(bus.read(map::TIMA), 255);

        timer.step();

        assert_eq!(bus.read(map::DIV), 7);
        assert_eq!(bus.read(map::TIMA), 255);

        timer.step();

        assert_eq!(bus.read(map::DIV), 8);
        assert_eq!(bus.read(map::TIMA), 254);

        assert_eq!(timer.get_interrupt(), true);
    }
}
