use std::cell::RefCell;
use std::panic;
use std::path::Path;
use std::rc::Rc;

use backtrace::Backtrace;
use log::info;
use minifb;

use super::cartridge::load_cartridge_from_file;
use super::cartridge::CartridgePlatform;
use super::config::Config;
use super::cpu::IntReg;
use super::cpu::CPU;
use super::debug::dump_cpu_record;
use super::debug::Inspector;
use super::graphics::gpu::GPU;
use super::joypad::Joypad;
use super::joypad::JoypadKey;
use super::mmu::Mmunit;
use super::set_global_term;
use super::timer::Timer;
use super::Term;
use super::{SCREEN_H, SCREEN_W};

pub struct Emulator {
    config: Config,
    cpu: CPU,
    gpu: Rc<RefCell<GPU>>,
    pub mmu: Rc<RefCell<Mmunit>>,
    inspector: Inspector,
}

impl Emulator {
    pub fn new(config: Config) -> Self {
        let path = Path::new(config.get_file_path());
        let cart = load_cartridge_from_file(path);
        let term = match cart.get_meta().get_platform() {
            CartridgePlatform::GBC => Term::GBC,
            CartridgePlatform::GBC_ONLY => Term::GBC,
            CartridgePlatform::SGB => Term::SGB,
            _ => Term::GB,
        };

        set_global_term(term);

        let intf = Rc::new(RefCell::new(IntReg::new()));

        let gpu = Rc::new(RefCell::new(GPU::new(intf.clone())));
        let joypad = Joypad::new(intf.clone());
        let timer = Timer::new(intf.clone());

        let mmu = Rc::new(RefCell::new(Mmunit::new(
            cart,
            gpu.clone(),
            joypad,
            timer,
            intf.clone(),
        )));
        let cpu = CPU::new(mmu.clone(), true);
        info! {"Emulator new {:?}", cpu.get_reg_snapshot()};

        Self {
            config,
            cpu,
            gpu,
            mmu,
            inspector: Inspector::new(),
        }
    }

    fn next(&mut self) -> u32 {
        if self.inspector.should_enter_trap() {
            // println!("{:?}", self.cpu.reg);
            self.inspector.break_here(&self.cpu, self.gpu.clone());
        }
        let cycles = self.cpu.next();
        self.mmu.borrow_mut().next(cycles);
        cycles
    }

    // fn _run(mut self) -> ! {
    //     let event_loop = EventLoop::new();
    //     let mut input = WinitInputHelper::new();
    //     let window = {
    //         let size = LogicalSize::new(SCREEN_W as f64, SCREEN_H as f64); // PhysicalSize
    //         WindowBuilder::new()
    //             .with_title(format!("Gameboy - {}", "RED").as_str())
    //             .with_inner_size(size)
    //             .with_min_inner_size(size)
    //             .build(&event_loop)
    //             .unwrap()
    //     };

    //     let mut pixels = {
    //         let window_size = window.inner_size();
    //         let surface_texture =
    //             SurfaceTexture::new(window_size.width, window_size.height, &window);
    //         Pixels::new(SCREEN_W as u32, SCREEN_H as u32, surface_texture).unwrap()
    //     };

    //     let mut last_update: std::time::Instant = std::time::Instant::now();

    //     event_loop.run(move |event, _, control_flow| {
    //         // Draw the current frame
    //         // println!("render event {:?}", event);

    //         self.next();
    //         let sixteen_ms = std::time::Duration::from_millis(16);
    //         let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
    //         if duration_since_last_update < sixteen_ms {
    //             // std::thread::sleep(sixteen_ms - duration_since_last_update);
    //         }
    //         // println!("sleep out");

    //         if let Event::RedrawRequested(_) = event {
    //            if self.mmu.borrow().gpu.borrow().should_updated() {
    //                self.mmu.borrow_mut().gpu.borrow_mut().reset_updated();
    //                 // println!("inner");
    //                 // println!("render");
    //                 let mut frame = pixels.get_frame();

    //                 let mut should_render = false;
    //                 for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
    //                     let x = (i % SCREEN_W as usize) as usize;
    //                     let y = (i / SCREEN_W as usize) as usize;

    //                     let w = self.mmu.borrow().gpu.borrow().get_data()[y][x];

    //                     let b = u32::from(w[0]) << 16;
    //                     let g = u32::from(w[1]) << 8;
    //                     let r = u32::from(w[2]);
    //                     // let a = 0xff00_0000;

    //                     let rgba = [w[2], w[1] as u8, w[0], 0xff];
    //                     if w != [255, 255, 255] {
    //                         // println!("rgba is {:?}", w);
    //                     }
    //                     if rgba != pixel {
    //                         pixel.copy_from_slice(&rgba);
    //                         should_render = true;
    //                     }
    //                     // let start = SystemTime::now();
    //                     // let since_the_epoch = start
    //                     //     .duration_since(UNIX_EPOCH)
    //                     //     .expect("Time went backwards");
    //                     // println!("{:?}", since_the_epoch);
    //                     // thread::sleep(time::Duration::from_millis(30));
    //                 }
    //                 // for l in self.mmu.borrow().gpu.data.iter() {
    //                 //     for w in l.iter() {
    //                 //         let b = u32::from(w[0]) << 16;
    //                 //         let g = u32::from(w[1]) << 8;
    //                 //         let r = u32::from(w[2]);
    //                 //         let a = 0xff00_0000;

    //                 //         frame[i] = a | b | g | r;
    //                 //         // 255 << 16 | 0xff00_0000 | 255 << 8 | 255
    //                 //         // if window_buffer[i] != 255 && window_buffer[i] != 4294967295 {
    //                 //         //     println!("fuck {}", window_buffer[i]);
    //                 //         // }
    //                 //         i += 1;
    //                 //     }
    //                 // }

    //                 if should_render {
    //                     // println!("render");
    //                     if pixels
    //                         .render()
    //                         .map_err(|e| println!("pixels.render() failed: {}", e))
    //                         .is_err()
    //                     {
    //                         *control_flow = ControlFlow::Exit;
    //                         return;
    //                     }
    //                 } else {
    //                     // println!("not render");
    //                 }
    //             }
    //         }

    //         if input.update(&event) {
    //             if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
    //                 *control_flow = ControlFlow::Exit;
    //                 return;
    //             }

    //             // Resize the window
    //             // if let Some(size) = input.window_resized() {
    //             //     pixels.resize_surface(size.width, size.height);
    //             // }

    //             let keys = vec![
    //                 (VirtualKeyCode::D, JoypadKey::Right),
    //                 (VirtualKeyCode::W, JoypadKey::Up),
    //                 (VirtualKeyCode::A, JoypadKey::Left),
    //                 (VirtualKeyCode::S, JoypadKey::Down),
    //                 (VirtualKeyCode::J, JoypadKey::A),
    //                 (VirtualKeyCode::K, JoypadKey::B),
    //                 (VirtualKeyCode::N, JoypadKey::Select),
    //                 (VirtualKeyCode::M, JoypadKey::Start),
    //             ];

    //             for (rk, vk) in &keys {
    //                 if input.key_pressed(*rk) {
    //                     self.mmu.borrow_mut().joypad.keydown(vk.clone());
    //                     println!("is pressed {:?}", rk);
    //                     // break
    //                 }
    //                 if input.key_released(*rk) {
    //                     self.mmu.borrow_mut().joypad.keyup(vk.clone());
    //                 }
    //             }
    //             // (minifb::Key::Right, JoypadKey::Right),
    //             // (minifb::Key::Up, JoypadKey::Up),
    //             // (minifb::Key::Left, JoypadKey::Left),
    //             // (minifb::Key::Down, JoypadKey::Down),
    //             // (minifb::Key::Z, JoypadKey::A),
    //             // (minifb::Key::X, JoypadKey::B),
    //             // (minifb::Key::Space, JoypadKey::Select),
    //             // (minifb::Key::Enter, JoypadKey::Start),
    //             window.request_redraw();
    //         }

    //         last_update = std::time::Instant::now();
    //     });
    // }

    fn _run(&mut self) {
        let c_scale = 2;
        let mut option = minifb::WindowOptions::default();
        option.resize = true;
        option.scale = match c_scale {
            1 => minifb::Scale::X1,
            2 => minifb::Scale::X2,
            4 => minifb::Scale::X4,
            8 => minifb::Scale::X8,
            _ => panic!("Supported scale: 1, 2, 4 or 8"),
        };
        let mut window = minifb::Window::new(
            format!("Gameboy - {}", "RED").as_str(),
            SCREEN_W,
            SCREEN_H,
            option,
        )
        .unwrap();
        let mut window_buffer = vec![0x00; SCREEN_W * SCREEN_H];
        window
            .update_with_buffer(window_buffer.as_slice(), SCREEN_W, SCREEN_H)
            .unwrap();

        loop {
            if !window.is_open() {
                break;
            }

            self.next();

            if self.mmu.borrow().gpu.borrow().should_updated() {
                self.mmu.borrow_mut().gpu.borrow_mut().reset_updated();
                // println!("{:?}", self.mmu.borrow().gpu.data);
                let mut i: usize = 0;

                for l in self.mmu.borrow().gpu.borrow().get_data().iter() {
                    for w in l.iter() {
                        let b = u32::from(w[0]) << 16;
                        let g = u32::from(w[1]) << 8;
                        let r = u32::from(w[2]);
                        let a = 0xff00_0000;

                        window_buffer[i] = a | b | g | r;

                        i += 1;
                    }
                }

                // let start = SystemTime::now();
                // let since_the_epoch = start
                //     .duration_since(UNIX_EPOCH)
                //     .expect("Time went backwards");
                // println!("{:?}", since_the_epoch);
                window
                    .update_with_buffer(window_buffer.as_slice(), SCREEN_W, SCREEN_H)
                    .unwrap();
            }

            if !self.cpu.flip() {
                continue;
            }

            let keys = vec![
                (minifb::Key::D, JoypadKey::Right),
                (minifb::Key::W, JoypadKey::Up),
                (minifb::Key::A, JoypadKey::Left),
                (minifb::Key::S, JoypadKey::Down),
                (minifb::Key::J, JoypadKey::A),
                (minifb::Key::K, JoypadKey::B),
                (minifb::Key::N, JoypadKey::Select),
                (minifb::Key::M, JoypadKey::Start),
            ];
            for (rk, vk) in &keys {
                if window.is_key_down(*rk) {
                    self.mmu.borrow_mut().joypad.keydown(vk.clone());
                    // It's so important
                    break;
                } else {
                    self.mmu.borrow_mut().joypad.keyup(vk.clone());
                }
            }
        }
    }

    fn set_panic_hook() {
        panic::set_hook(Box::new(|panic_info| {
            let bt = Backtrace::new();
            let mut msg = vec![];
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                msg.push(format!("panic occurred: {:?}", s));
            } else {
                msg.push("panic occurred: (no detail)".to_string());
            }
            if let Some(location) = panic_info.location() {
                msg.push(format!(
                    "in file '{}' at line {}",
                    location.file(),
                    location.line(),
                ));
            } else {
                msg.push("but can't get location information...".to_string());
            }
            println!("{}", msg.join(" "));
            println!("The full backtrace is {:?}", bt);
            dump_cpu_record(Path::new("./coredump")); // TODO: file name
        }));
    }

    pub fn run(&mut self) {
        self.inspector.start_monitor();
        Self::set_panic_hook();
        self._run();
    }
}
