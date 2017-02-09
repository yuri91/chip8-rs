extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::render::Renderer;

mod chip8;

use std::fs;
use std::io::Read;

use std::thread;
use std::time;

use std::collections::HashMap;

struct SDLFrontend {
    chip8: chip8::Cpu,
    sdl_context: Sdl,
    renderer: Renderer<'static>,
    keymap: HashMap<String,usize>
}

impl SDLFrontend {
    pub fn init() -> SDLFrontend {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("SDL frontend", 768, 384)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut keymap = HashMap::new();
        keymap.insert("1".to_string(),0x1usize);
        keymap.insert("2".to_string(),0x2usize);
        keymap.insert("3".to_string(),0x3usize);
        keymap.insert("4".to_string(),0xCusize);
        keymap.insert("Q".to_string(),0x4usize);
        keymap.insert("W".to_string(),0x5usize);
        keymap.insert("E".to_string(),0x6usize);
        keymap.insert("R".to_string(),0xDusize);
        keymap.insert("A".to_string(),0x7usize);
        keymap.insert("S".to_string(),0x8usize);
        keymap.insert("D".to_string(),0x9usize);
        keymap.insert("F".to_string(),0xEusize);
        keymap.insert("Z".to_string(),0xAusize);
        keymap.insert("X".to_string(),0x0usize);
        keymap.insert("C".to_string(),0xBusize);
        keymap.insert("V".to_string(),0xFusize);

        SDLFrontend {
            chip8: chip8::Cpu::new(),
            sdl_context: sdl_context,
            renderer: window.renderer().build().unwrap(),
            keymap: keymap
        }
    }
    pub fn load_program(&mut self, path: &str) {
        let mut f = fs::File::open(path).expect("file does not exists");
        let mut program = Vec::new();
        f.read_to_end(&mut program).expect("cannot read file");
        self.chip8.load(program.as_slice());
    }
    fn update_screen(&mut self) {

        let w = 64usize;
        let h = 32usize;
        let scale = 768usize/w;

        self.renderer.set_draw_color(Color::RGB(255, 255, 255));
        self.renderer.clear();
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        for j in 0..h {
            for i in 0..w {
                if self.chip8.video_ram()[i+64*j] {
                    self.renderer.fill_rect(Rect::new((i*scale) as i32,(j*scale) as i32,scale as u32,scale as u32)).unwrap();
                }
            }
        }
        self.renderer.present();
    }
    fn update_keys(&mut self) -> bool {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return false;
                },
                Event::KeyDown { keycode: Some(k), .. } => {
                    if let Some(p) = self.keymap.get(&k.name()) {
                        self.chip8.keys_pressed()[*p] = true;
                    }
                },
                Event::KeyUp { keycode: Some(k), .. } => {
                    if let Some(p) = self.keymap.get(&k.name()) {
                        self.chip8.keys_pressed()[*p] = false;
                    }
                },
                _ => {}
            }
        }
        return true;
    }

    pub fn run(&mut self) {
        let mut running = true;
        let inst_per_frame = 20;
        let dur = time::Duration::new(0,1_000_000_000/60);
        let clock = time::SystemTime::now();
        while running {
            let start = clock.elapsed().unwrap();
            self.chip8.run(inst_per_frame);
            running = self.update_keys();
            self.update_screen();
            self.chip8.tick();
            let elapsed = clock.elapsed().unwrap() - start;

            if elapsed < dur {
                thread::sleep(dur-elapsed);
            }
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("must provide a ROM file!");

    let mut sdl_frontend = SDLFrontend::init();

    sdl_frontend.load_program(&path);
    sdl_frontend.run();

}
