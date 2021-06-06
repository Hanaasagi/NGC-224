# NGC-224

A Gameboy emulator built in Rust.


![](https://github.com/Hanaasagi/NGC-224/blob/master/.screenshot/pokemon.gif)


## Features

- [x] CPU and instructions
- [x] PPU(GPU)
- [x] Cartridge(MBC)
  - [x] MBC1
  - [x] MBC2
  - [x] MBC3
  - [x] ROM ONLY
- [x] Memory Management
- [x] Joypad Control
- [ ] Audio
- [ ] Serial
- [ ] CGB-MODE


## Building and Installation

```Bash
$ git clone https://github.com/Hanaasagi/NGC-224.git
$ cd NGC-224
$ cargo run -- -p <your rom path>
```


## KeyBoard Mapping

| Keyboard     | Gameboy |
| --------     | ------- |
| <kbd>M</kbd> | Start   |
| <kbd>N</kbd> | Select  |
| <kbd>w</kbd> | Up      |
| <kbd>s</kbd> | Down    |
| <kbd>a</kbd> | Left    |
| <kbd>d</kbd> | Right   |
| <kbd>j</kbd> | A       |
| <kbd>k</kbd> | B       |



## Bug Report

If the program panic, please send the `coredump` file in current directory and tell me which rom you are playing. Additionally, the process receive a `USR1` signal for starting a stepping debug.



## Reference

Thanks for the article and open source project below.

- [Gameboy Development Wiki](https://gbdev.gg8.se/wiki)
- [GameBoy 仿真器 / accu.cc](http://accu.cc/content/gameboy/preface/)
- [Gameboy Emulation / codeslinger.co.uk](http://www.codeslinger.co.uk/pages/projects/gameboy/dma.html)
- [mvdnes/rboy](https://github.com/mvdnes/rboy)
- [mohanson/gameboy](https://github.com/mohanson/gameboy)
- [HFO4/gameboy.live](https://github.com/HFO4/gameboy.live)
