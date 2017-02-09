mod cpu;
use self::cpu::Cpu;

use std::fs::File;
use std::io::{self,Read};

use std::thread::sleep;
use std::time::{Duration,SystemTime};

pub trait Keyboard {
    fn update_keys(&mut self, keys: &mut[bool]) -> bool;
}

pub trait Display {
    fn update_screen(&mut self, screen: &[bool]);
}

pub struct Chip8<Frontend: Keyboard+Display> {
    cpu: Cpu,
    frontend: Frontend
}
use std;
impl <Frontend:Display+Keyboard> Chip8<Frontend> {

    pub fn new(f: Frontend) -> Chip8<Frontend> {
        Chip8 {
            cpu: Cpu::new(),
            frontend: f
        }
    }
    pub fn load_program(&mut self, path: &str) {
        let mut f = File::open(path).expect("file does not exists");
        let mut program = Vec::new();
        f.read_to_end(&mut program).expect("cannot read file");
        self.cpu.load(program.as_slice());
    }
    pub fn run(&mut self) {
        let mut running = true;
        let now = SystemTime::now();
        let mut last_cpu = now.elapsed().unwrap();
        let mut last_timer = last_cpu;
        let inst_per_frame = 12;
        let cpu_time = Duration::new(0,1_000_000_000/(60*inst_per_frame));
        let timer_time = Duration::new(0,1_000_000_000/60);
        while running {
            let elapsed = now.elapsed().unwrap();
            let diff_cpu = elapsed - last_cpu;
            if (diff_cpu >= cpu_time) {
                self.cpu.fetch();
                self.cpu.exec();
                last_cpu = elapsed;
            }
            let diff_timer = elapsed - last_timer;
            if (diff_timer >= timer_time) {
                running = self.frontend.update_keys(self.cpu.keys_pressed());
                self.frontend.update_screen(self.cpu.video_ram());

                self.cpu.tick();
                last_timer = elapsed;
            }
        }
    }
}
