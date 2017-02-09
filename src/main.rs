extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::render::Renderer;

mod chip8;
use chip8::{Chip8,Display,Keyboard};

use std::collections::HashMap;

struct SDLFrontend {
    sdl_context: Sdl,
    renderer: Renderer<'static>,
    keymap: HashMap<String,usize>
}

impl SDLFrontend {
    fn init() -> SDLFrontend {
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
            sdl_context: sdl_context,
            renderer: window.renderer().build().unwrap(),
            keymap: keymap
        }
    }
}

impl Display for SDLFrontend {
    fn update_screen(&mut self, screen: &[bool]) {

        let w = 64usize;
        let h = 32usize;
        let scale = 768usize/w;

        self.renderer.set_draw_color(Color::RGB(255, 255, 255));
        self.renderer.clear();
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        for j in 0..h {
            for i in 0..w {
                if screen[i+64*j] {
                    self.renderer.fill_rect(Rect::new((i*scale) as i32,(j*scale) as i32,scale as u32,scale as u32));
                }
            }
        }
        self.renderer.present();
    }
}

impl Keyboard for SDLFrontend {
    fn update_keys(&mut self, keys: &mut[bool]) -> bool {
        
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return false;
                },
                Event::KeyDown { keycode: Some(k), .. } => {
                    if let Some(p) = self.keymap.get(&k.name()) {
                        keys[*p] = true;
                    }
                },
                Event::KeyUp { keycode: Some(k), .. } => {
                    if let Some(p) = self.keymap.get(&k.name()) {
                        keys[*p] = false;
                    }
                },
                _ => {}
            }
        }
        return true;
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("must provide a ROM file!");

    let sdl_frontend = SDLFrontend::init();

    let mut chip8 = Chip8::new(sdl_frontend);

    chip8.load_program(&path);
    chip8.run();

}
