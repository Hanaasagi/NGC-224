use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use log::error;
use rustyline::Editor;

use super::cpu::Register;
use super::cpu::CPU;
use super::graphics::gpu::GPU;

const RECORE_LIMIT: usize = 512;

#[derive(Debug, Clone)]
pub struct CPUDebugInfo {
    reg: Register,
    opcode: u8,
    is_ext_opcode: bool,
}

impl CPUDebugInfo {
    pub fn new(reg: Register, opcode: u8, is_ext_opcode: bool) -> Self {
        Self {
            reg,
            opcode,
            is_ext_opcode,
        }
    }
}

lazy_static! {
    static ref CPU_RECORD: Mutex<VecDeque<CPUDebugInfo>> = Mutex::new(VecDeque::new());
}

pub fn insert_cpu_record(record: CPUDebugInfo) {
    let data = CPU_RECORD.lock();
    if data.is_err() {
        error!("insert the cpu debug info failed {:?}, skip", data.err());
        return;
    }
    let mut q = data.unwrap();

    if q.len() >= RECORE_LIMIT {
        q.pop_front();
    }
    q.push_back(record);
}

pub fn dump_cpu_record(file_path: impl AsRef<Path>) {
    let f = File::create(file_path).unwrap();
    let mut f = LineWriter::new(f);
    let data = CPU_RECORD.lock().unwrap();
    for line in data.iter() {
        f.write(format!("{:?}\n", line).as_bytes())
            .expect("write file failed");
    }
    f.flush().expect("flush file failed");
}

pub struct Inspector {
    rl: Editor<()>,
    flag: Arc<AtomicBool>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            rl: Editor::new(),
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_monitor(&self) {
        signal_hook::flag::register(signal_hook::consts::SIGUSR1, Arc::clone(&self.flag)).unwrap();
    }

    pub fn should_enter_trap(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn break_here(&mut self, cpu: &CPU, _: Rc<RefCell<GPU>>) {
        loop {
            let readline = self.rl.readline(">>> ");
            match readline {
                Ok(line) if line.starts_with("help") => {
                    println!("help here, todo");
                }
                Ok(line) if line.starts_with("next") => {
                    self.rl.add_history_entry(line.as_str());
                    break;
                }
                Ok(line) if line.starts_with("detach") => {
                    self.rl.add_history_entry(line.as_str());
                    self.flag.store(false, Ordering::Relaxed);
                    break;
                }
                Ok(line) if line.starts_with("var") => {
                    if let Some(obj) = line.split_ascii_whitespace().nth(1) {
                        self.rl.add_history_entry(line.as_str());
                        match obj {
                            "cpu" => {
                                println!("cpu register is {:?}", cpu.get_reg_snapshot())
                            }
                            "gpu" => {
                                // TODO:
                            }
                            "opcode" => {
                                println!("next opcode is {:0x}", cpu.get_current_opcode())
                            }
                            _ => {
                                println!("unknown object")
                            }
                        }
                        continue;
                    }
                    println!("var command parse failed");
                }
                Ok(line) if line.starts_with("dump") => {
                    self.rl.add_history_entry(line.as_str());
                    dump_cpu_record(Path::new("./coredump"));
                }
                Ok(line) => {
                    println!("unknown command {}", line);
                }
                Err(_) => {
                    println!("aborted");
                    std::process::exit(0);
                }
            }
        }
    }
}
