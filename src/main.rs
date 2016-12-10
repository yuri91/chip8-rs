extern crate rand;

use std::fmt;
use std::fs::File;
use std::io::{self,Read};

#[derive(Copy, Clone, Debug)]
struct Opcode (u8,u8,u8,u8);

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

#[derive(Debug,Copy,Clone)]
enum KeyStatus {
    None,
    Waiting(u8),
}

#[derive(Debug,Clone)]
struct SpriteData {
    x: u8, y: u8,
    pixels: Vec<[bool;16]>
}
#[derive(Debug,Clone)]
enum ScreenCmd {
    None,
    Clear,
    Sprite(SpriteData)
}

trait DisplayInput {
    fn collision(&mut self, happened: bool);
}
trait DisplayOutput {
    fn clear(&mut self);
    fn flip(&mut self, sprite: SpriteData) -> bool;
}

#[derive(Clone,Debug)]
struct Cpu {
    ram: Vec<u8>,
    stack: Vec<u16>,
    v: [u8;16],
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,
    current: Opcode,
    key_status: KeyStatus,
    screen_cmd: ScreenCmd,
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            ram: vec![0;4096],
            stack: Vec::default(),
            v: [0;16],
            pc: 0x200,
            i: 0,
            dt: 0,
            st: 0,
            current: Opcode(0,0,0,0),
            key_status: KeyStatus::None,
            screen_cmd: ScreenCmd::None
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
        write!(f,"key_status: {:?}\n\t",self.key_status)?;
        write!(f,"screen_cmd: {:?}\n\t",self.screen_cmd)
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Default::default()
    }
    pub fn exec(&mut self) {
        let op = self.current;
        exec(self,op);
        println!("{}",&self);
    }
    pub fn fetch(&mut self) {
        self.current = Opcode::read(&self.ram[(self.pc as usize)..])
            .expect("wrapping memory!");
        self.pc += 2;
    }
    pub fn load(&mut self, program: &[u8]) {
        self.ram[0x200..0x200+program.len()].copy_from_slice(program);
    }
}
impl DisplayInput for Cpu {
    fn collision(&mut self, happened: bool) {
        self.v[0xF] = if happened { 1 } else { 0 };
    }
}

instruction_set! {
    cpu: Cpu;

    CLR     => 0x0,0x0,0xE,0x0 | {
        cpu.screen_cmd = ScreenCmd::Clear;
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
        cpu.v[x as usize] += nn(n1,n2);
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
    DRAW    => 0xD, _ , _ , _  | {},
    SKIPK   => 0xE, x ,0xE,0x9 | {},
    SKIPNK  => 0xE, x ,0xA,0x1 | {},
    MOVED   => 0xF, _ ,0x0,0x7 | {},
    KEYD    => 0xF, _ ,0x0,0xA | {},
    LOADD   => 0xF, _ ,0x1,0x5 | {},
    LOADS   => 0xF, _ ,0x1,0x8 | {},
    ADDI    => 0xF, _ ,0x1,0xE | {},
    LDSPR   => 0xF, _ ,0x2,0x9 | {},
    BCD     => 0xF, _ ,0x3,0x3 | {},
    STOR    => 0xF, _ ,0x5,0x5 | {},
    READ    => 0xF, _ ,0x6,0x5 | {},
    UNKNOWN =>  _ , _ , _ , _  | {}
}


fn main() {
    let mut f = File::open("test_roms/MAZE").expect("file does not exists");
    let mut program = Vec::new();
    f.read_to_end(&mut program).expect("cannot read file");

    println!("PROGRAM:");
    for c in program.chunks(2){
        println!("{}",Opcode::read(c).unwrap());
    }
    let mut cpu = Cpu::new();
    cpu.load(program.as_slice());
    let mut line = String::new();
    loop {
        cpu.fetch();
        cpu.exec();
        io::stdin().read_line(&mut line);
    }
}
