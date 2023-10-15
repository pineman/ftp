# Supposedly some ~~forth implementation~~ gameboy color emulator

## Running

`cargo run`

Running the disassembler:

`cargo run --bin objdump`

Running the debugger:

`cargo run --bin debug`

## Testing

`cargo test`

## References
### Opcodes

- [pastraiser.com — Gameboy CPU (LR35902) instruction set](<https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html>)
- [RGBDS docs — gbz80(7) — CPU opcode reference](https://rgbds.gbdev.io/docs/v0.6.1/gbz80.7/)
- [Dinu, Christian — Decoding Z80 Opcodes](http://z80.info/decoding.htm)

### Manuals

- :book: [Gameboy Programming Manual v1.1](https://ia803208.us.archive.org/9/items/GameBoyProgManVer1.1/GameBoyProgManVer1.1.pdf)
- :book: [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
- :book: [gekkio — Game Boy: Complete Technical Reference](<https://gekkio.fi/files/gb-docs/gbctr.pdf>)

### Misc

- [Game boy dev pandocs](https://gbdev.io/pandocs/)
- [Copetti — Game Boy Architecture](https://www.copetti.org/writings/consoles/game-boy/)
- [DMG-01: How to Emulate a Game Boy](https://rylev.github.io/DMG-01/public/book/cpu/introduction.html  )
- :movie_camera: [The Ultimate Game Boy Talk (33c3)](https://www.youtube.com/watch?v=HyzD8pNlpwI)
- :movie_camera: The Game Boy, a hardware autopsy
    ([Part 1](https://www.youtube.com/watch?v=RZUDEaLa5Nw),
     [Part 1.5](https://www.youtube.com/watch?v=t0V-D2YMhrs),
     [Part 2](https://www.youtube.com/watch?v=ecTQVa42sJc))
