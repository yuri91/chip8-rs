extern crate rand;

use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct Opcode (u8,u8,u8,u8);

impl Opcode {
    pub fn read(mem: &[u8]) -> Option<Opcode> {
        if mem.len() < 2 {
            None
        } else {
            let b1 = mem[0];
            let b2 = mem[1];
            Some(Opcode(b1>>4,b1&0x0F,b2>>4,b2&0x0F))
        }
    }
}

#[inline]
fn nn(n1:u8,n2:u8) -> u8 {
    n1<<4 | n2
}
#[inline]
fn nnn(n1:u8,n2:u8,n3:u8) -> u16 {
    (nn(n1,n2) as u16) << 4 | (n3 as u16)
}

impl Opcode {
    fn x(&self) -> u8 { self.1 }
    fn y(&self) -> u8 { self.2 }
    fn n(&self) -> u8 { self.3 }
    fn nn(&self) -> u8 { nn(self.2,self.3) }
    fn nnn(&self) -> u16 { nnn(self.1,self.2,self.3) }
}


macro_rules! instruction_set{
    (
        $cpu:ident:$ty:ty;
        $($name:ident => $($pat:pat),* | $body:expr),*
    ) => {
        #[inline]
        fn exec($cpu: &mut $ty, op: Opcode) {
            match op {
            $(
                Opcode($($pat),*) => $body
            ),*
            }
        }
        impl fmt::Display for Opcode {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match *self {
                $(
                    Opcode($($pat),*) => {
                        write!(f, "0x{:X}{:X}{:X}{:X}\t{}\t",
                               self.0,self.1,self.2,self.3,stringify!($name))?;
                        write!(f, "x: {:X}, y: {:X}, n: {}, nn: {}, nnn: {:#X}",
                               self.x(),self.y(),self.n(),self.nn(),self.nnn())
                    }
                ),*
                }
            }
        }
    };
}

#[derive(Clone,Debug)]
pub struct Cpu {
    ram: Vec<u8>,
    video_ram: Vec<bool>,
    stack: Vec<u16>,
    v: [u8;16],
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,
    current: Opcode,
    keys_pressed: Vec<bool>,
}

const font : [u8;16*5] = [
      0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
      0x20, 0x60, 0x20, 0x20, 0x70, // 1
      0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
      0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
      0x90, 0x90, 0xF0, 0x10, 0x10, // 4
      0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
      0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
      0xF0, 0x10, 0x20, 0x40, 0x40, // 7
      0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
      0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
      0xF0, 0x90, 0xF0, 0x90, 0x90, // A
      0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
      0xF0, 0x80, 0x80, 0x80, 0xF0, // C
      0xE0, 0x90, 0x90, 0x90, 0xE0, // D
      0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
      0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            ram: vec![0;4096],
            video_ram: vec![false;64*32],
            stack: Vec::default(),
            v: [0;16],
            pc: 0x0,
            i: 0,
            dt: 0,
            st: 0,
            current: Opcode(0,0,0,0),
            keys_pressed: vec![false;16],
        }
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Cpu info:\n\t")?;
        for (i,reg) in self.v.iter().enumerate() {
            write!(f,"v{:X}: {:#X}\n\t",i,reg)?;
        }
        write!(f,"pc: {:#X} => {}\n\t",self.pc,self.current)?;
        write!(f,"I: {:#X} => {:#X}\n\t",self.i,self.ram[self.i as usize])?;
        write!(f,"dt: {} , st: {}\n\t",self.dt,self.st)?;
        write!(f,"stack: {:?}\n\t",&self.stack)?;
        write!(f,"keys_pressed: {:?}\n\t",self.keys_pressed)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu::default();
        cpu.pc = 0x200u16;
        cpu.ram[0..16*5].copy_from_slice(&font);
        cpu
    }
    pub fn exec(&mut self) {
        let op = self.current;
        exec(self,op);
        //println!("{}",&self);
    }
    pub fn fetch(&mut self) {
        self.current = Opcode::read(&self.ram[(self.pc as usize)..])
            .expect("wrapping memory!");
        self.pc += 2;
    }
    pub fn tick(&mut self) {
        if self.st > 0 {
            self.st -= 1;
        }
        if self.dt > 0 {
            self.dt -= 1;
        }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.ram[0x200..0x200+program.len()].copy_from_slice(program);
    }
    pub fn video_ram(&self) -> &[bool] {
        self.video_ram.as_ref()
    }

    pub fn keys_pressed(&mut self) -> &mut[bool] {
        self.keys_pressed.as_mut()
    }
}

instruction_set! {
    cpu: Cpu;

    CLR     => 0x0,0x0,0xE,0x0 | {
        for p in &mut cpu.video_ram {
            *p = false;
        }
    },
    RTS     => 0x0,0x0,0xE,0xE | {
        cpu.pc = cpu.stack.pop().expect("stack overflow!");
    },
    JUMP    => 0x1, n1, n2, n3 | {
        cpu.pc = nnn(n1,n2,n3);
    },
    CALL    => 0x2, n1, n2, n3 | {
        cpu.stack.push(cpu.pc);
        cpu.pc = nnn(n1,n2,n3);
    },
    SKE     => 0x3, x , n1, n2 | {
        if cpu.v[x as usize] == nn(n1,n2) {
            cpu.pc += 2;
        }
    },
    SKNE    => 0x4, x , n1, n2 | {
        if cpu.v[x as usize] == nn(n1,n2) {
            cpu.pc += 2;
        }
    },
    SKRE    => 0x5, x , y ,0x0 | {
        if cpu.v[x as usize] == cpu.v[y as usize] {
            cpu.pc += 2;
        }
    },
    LOAD    => 0x6, x , n1, n2 | {
        cpu.v[x as usize] = nn(n1,n2);
    },
    ADD     => 0x7, x , n1, n2 | {
        cpu.v[x as usize] = cpu.v[x as usize].wrapping_add(nn(n1,n2));
    },
    MOVE    => 0x8, x , y ,0x0 | {
        cpu.v[x as usize] = cpu.v[y as usize];
    },
    OR      => 0x8, x , y ,0x1 | {
        cpu.v[x as usize] |= cpu.v[y as usize];
    },
    AND     => 0x8, x , y ,0x2 | {
        cpu.v[x as usize] &= cpu.v[y as usize];
    },
    XOR     => 0x8, x , y ,0x3 | {
        cpu.v[x as usize] ^= cpu.v[y as usize];
    },
    ADDR    => 0x8, x , y ,0x4 | {
        let (res, of) = cpu.v[x as usize].overflowing_add(cpu.v[y as usize]);
        cpu.v[x as usize] = res;
        cpu.v[0xF] = of as u8;
    },
    SUB     => 0x8, x , y ,0x5 | {
        let (res, of) = cpu.v[x as usize].overflowing_sub(cpu.v[y as usize]);
        cpu.v[x as usize] = res;
        cpu.v[0xF] = (!of) as u8;
    },
    SHR     => 0x8, x , y ,0x6 | {
        cpu.v[0xF] = cpu.v[x as usize] & 0x01;
        cpu.v[x as usize] >>= 1;
    },
    SUB     => 0x8, x , y ,0x7 | {
        let (res, of) = cpu.v[x as usize].overflowing_sub(cpu.v[y as usize]);
        cpu.v[x as usize] = res;
        cpu.v[0xF] = (!of) as u8;
    },
    SHL     => 0x8, x , y ,0xE | {
        cpu.v[0xF] = cpu.v[x as usize] & 0x80;
        cpu.v[x as usize] <<= 1;
    },
    SKRNE   => 0x9, x , y ,0x0 | {
        if cpu.v[x as usize] == cpu.v[y as usize] {
            cpu.pc += 2;
        }
    },
    LOADI   => 0xA, n1, n2, n3 | {
        cpu.i = nnn(n1,n2,n3);
    },
    JUMPI   => 0xB, n1, n2, n3 | {
        cpu.pc = cpu.v[0x0] as u16 + nnn(n1,n2,n3);
    },
    RAND    => 0xC, x , n1, n2 | {
        cpu.v[x as usize] = nn(n1,n2) & rand::random::<u8>();
    },
    DRAW    => 0xD, x , y , n  | {
        cpu.v[0xF] = 0;
        for row in 0..n {
            let data = cpu.ram[(cpu.i+row as u16) as usize];
            for col in 0..8 {
                let px = (cpu.v[x as usize] as usize + (col as usize)) % 64;
                let py = (cpu.v[y as usize] as usize + (row as usize)) % 32;
                let pos = px + 64*py;
                if data & (0x80>>col) != 0 {
                    if cpu.video_ram[pos] {
                        cpu.v[0xF] = 1;
                        cpu.video_ram[pos] = false;
                    } else {
                        cpu.video_ram[pos] = true;
                    }
                }
            }
        }
    },
    SKP   => 0xE, x ,0x9,0xE | {
        if cpu.keys_pressed[x as usize] {
            cpu.pc +=2;
        }
    },
    SKNP  => 0xE, x ,0xA,0x1 | {
        if !cpu.keys_pressed[x as usize] {
            cpu.pc +=2;
        }
    },
    MOVED   => 0xF, x ,0x0,0x7 | {
        cpu.v[x as usize] = cpu.dt;
    },
    KEYD    => 0xF, x ,0x0,0xA | {
        if let Some(p) = cpu.keys_pressed.iter().position(|&x| x) {
            cpu.v[x as usize] = p as u8;
        } else {
            cpu.pc -=2;
        }
    },
    LOADD   => 0xF, x ,0x1,0x5 | {
        cpu.dt = cpu.v[x as usize];
    },
    LOADS   => 0xF, x ,0x1,0x8 | {
        cpu.st = cpu.v[x as usize];
    },
    ADDI    => 0xF, x ,0x1,0xE | {
         cpu.i += cpu.v[x as usize] as u16;
    },
    LDSPR   => 0xF, x ,0x2,0x9 | {
        cpu.i = (cpu.v[x as usize] as u16) * 5;
    },
    BCD     => 0xF, x ,0x3,0x3 | {
        let mut k = cpu.v[x as usize];
        cpu.ram[(cpu.i+2) as usize] = k % 10;
        k /= 10;
        cpu.ram[(cpu.i+1) as usize] = k % 10;
        k /= 10;
        cpu.ram[cpu.i as usize] = k % 10;
    },
    STOR    => 0xF, x ,0x5,0x5 | {
        for r in 0..x {
            cpu.ram[(cpu.i as usize)+(r as usize)] = cpu.v[r as usize];
        }
    },
    READ    => 0xF, x ,0x6,0x5 | {
        for r in 0..x {
            cpu.v[r as usize] = cpu.ram[(cpu.i as usize)+(r as usize)];
        }
    },
    UNKNOWN =>  n1, n2, n3, n4 | {
        println!("{}",Opcode(n1,n2,n3,n4));
        panic!("unknown instruction, exiting");
    }
}
