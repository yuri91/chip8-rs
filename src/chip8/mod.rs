mod cpu;
use self::cpu::Cpu;

use std::fs::File;
use std::io::{self,Read};

use std::thread::sleep;
use std::time::Duration;

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
        let mut tick = 0u32;
        while running {
            running = self.frontend.update_keys(self.cpu.keys_pressed());

            self.frontend.update_screen(self.cpu.video_ram());
            self.cpu.fetch();
            self.cpu.exec();
            tick = tick.wrapping_add(1);
            if tick%9 == 0 {
                self.cpu.tick();
            }
            //let duration = Duration::new(0,1000_000_000/540);
            //sleep(duration);
            println!("{}",&self.cpu);
            let mut l = String::new();
            std::io::stdin().read_line(&mut l);
        }
    }
}
