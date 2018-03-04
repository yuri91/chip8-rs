#![allow(unused_variables)]
#![no_std]

use core::fmt;

#[derive(Copy, Clone, Debug)]
struct Opcode (u16);

impl Opcode {
    pub fn read(mem: &[u8]) -> Option<Opcode> {
        if mem.len() < 2 {
            None
        } else {
            let b1 = (mem[0] as u16) << 8;
            let b2 = mem[1] as u16;
            Some(Opcode(b1|b2))
        }
    }
}

impl Opcode {
    fn group(&self) -> u8 { (self.0 >> 12 & 0x0f) as u8 }
    fn x(&self) -> u8 { ((self.0 >> 8) & 0x0f) as u8}
    fn y(&self) -> u8 { ((self.0 >> 4) & 0x0f) as u8}
    fn n(&self) -> u8 { (self.0 & 0x0f) as u8}
    fn nn(&self) -> u8 { (self.0 & 0xff) as u8}
    fn nnn(&self) -> u16 { (self.0 & 0x0fff) }
}

const FONTS : [u8;16*5] = [
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

pub struct Memory {
    pub ram: [u8; 4096],
    pub video: [bool; 64*32],
    pub keys: [bool; 16],
}
impl Memory {
    pub fn new() -> Memory {
        let mut ram = [0; 4096];
        ram[0..16*5].copy_from_slice(&FONTS);
        Memory {
            ram,
            video: [false; 64*32],
            keys: [false; 16],
        }
    }
    pub fn load_program(&mut self, program: &[u8]) {
        self.ram[0x200..0x200+program.len()].copy_from_slice(program);
    }
    pub fn reset(&mut self) {
        self.ram[0..16*5].copy_from_slice(&FONTS);
        for i in self.video[..].iter_mut() {
            *i = false;
        }
        for i in self.keys[..].iter_mut() {
            *i = false;
        }
    }
}

#[derive(Clone, Debug)]
struct Stack {
    mem: [u16; 16],
    next: u8,
}
impl Stack {
    fn new() -> Stack {
        Stack {
            mem: [0; 16],
            next: 0,
        }
    }
    fn push(&mut self, i: u16) -> Result<(), ()> {
        if self.next < 16 {
            self.mem[self.next as usize] = i;
            self.next += 1;
            Ok(())
        } else {
            Err(())
        }
    }
    fn pop(&mut self) -> Result<u16, ()> {
        if self.next > 0 {
            self.next -= 1;
            Ok(self.mem[self.next as usize])
        } else {
            Err(())
        }
    }
    fn clear(&mut self) {
        self.next = 0;
    }
}

#[derive(Clone,Debug)]
pub struct Cpu {
    stack: Stack,
    v: [u8;16],
    pc: u16,
    i: u16,
    dt: u8,
    st: u8,
    current: Opcode,
    rand: fn() -> u8, 
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"Cpu info:\n\t")?;
        for (i,reg) in self.v.iter().enumerate() {
            write!(f,"v{:X}: {:#X}\n\t",i,reg)?;
        }
        write!(f,"pc: {:#X} => {}\n\t",self.pc,self.current.0)?;
        write!(f,"I: {:#X}\n\t",self.i)?;
        write!(f,"dt: {} , st: {}\n\t",self.dt,self.st)?;
        write!(f,"stack: {:?}\n\t",&self.stack)
    }
}

impl Cpu {
    pub fn new(rand: fn()->u8) -> Cpu {
        Cpu {
            stack: Stack::new(),
            v: [0;16],
            pc: 0x200,
            i: 0,
            dt: 0,
            st: 0,
            current: Opcode(0),
            rand: rand,
        }
    }
    pub fn reset(&mut self) {
            self.stack.clear();
            self.v.copy_from_slice(&[0;16]);
            self.pc = 0x200u16;
            self.i = 0;
            self.dt = 0;
            self.st = 0;
            self.current = Opcode(0);
    }
    pub fn run(&mut self, memory: &mut Memory, cycles: usize) {
        for _ in 0..cycles {
            self.fetch(memory);
            self.exec(memory);
        }
    }
    pub fn tick(&mut self) {
        if self.st > 0 {
            self.st -= 1;
        }
        if self.dt > 0 {
            self.dt -= 1;
        }
    }
    fn exec(&mut self, mem: &mut Memory) {
        let op = self.current;
        exec(self, mem, op);
    }
    fn fetch(&mut self, mem: &Memory) {
        self.current = Opcode::read(&mem.ram[(self.pc as usize)..])
            .expect("wrapping memory!");
        self.pc += 2;
    }
}

fn exec(cpu: &mut Cpu, mem: &mut Memory, op: Opcode) {
    match op.group() {
        0x0 => {
            match op.nn() {
                0xE0 => clr(mem),
                0xEE => rts(cpu),
                _ => panic!("unknown instruction"),
            }
        },
        0x1 => {
            jump(cpu, op.nnn());
        },
        0x2 => {
            call(cpu, op.nnn());
        },
        0x3 => {
            ske(cpu, op.x(), op.nn());
        },
        0x4 => {
            skne(cpu, op.x(), op.nn());
        },
        0x5 => {
            if op.n() != 0 {
                panic!("unknown instruction");
            }
            skre(cpu, op.x(), op.y());
        },
        0x6 => {
            load(cpu, op.x(), op.nn());
        },
        0x7 => {
            add(cpu, op.x(), op.nn());
        },
        0x8 => {
            match op.n() {
                0x0 => mov(cpu, op.x(), op.y()),
                0x1 => or(cpu, op.x(), op.y()),
                0x2 => and(cpu, op.x(), op.y()),
                0x3 => xor(cpu, op.x(), op.y()),
                0x4 => addr(cpu, op.x(), op.y()),
                0x5 => sub(cpu, op.x(), op.y()),
                0x6 => shr(cpu, op.x(), op.y()),
                0x7 => subn(cpu, op.x(), op.y()),
                0xE => shl(cpu, op.x(), op.y()),
                _   => panic!("unknown instruction"),
            }
        },
        0x9 => {
            if op.n() != 0 {
                panic!("unknown instruction");
            }
            skrne(cpu, op.x(), op.y());
        },
        0xA => {
            loadi(cpu, op.nnn());
        },
        0xB => {
            jumpi(cpu, op.nnn());
        },
        0xC => {
            rand(cpu, op.x(), op.nn());
        },
        0xD => {
            draw(cpu, mem, op.x(), op.y(), op.n());
        },
        0xE => {
            match op.nn() {
                0x9E => skp(cpu, mem, op.x()),
                0xA1 => sknp(cpu, mem, op.x()),
                _ => panic!("unknown instruction"),
            }
        },
        0xF => {
            match op.nn() {
                0x07 => moved(cpu, op.x()),
                0x0A => keyd(cpu, mem, op.x()),
                0x15 => loadd(cpu, op.x()),
                0x18 => loads(cpu, op.x()),
                0x1E => addi(cpu, op.x()),
                0x29 => ldspr(cpu, op.x()),
                0x33 => bcd(cpu, mem, op.x()),
                0x55 => stor(cpu, mem, op.x()),
                0x65 => read(cpu, mem, op.x()),
                _ => panic!("unknown instruction"),
            }
        },
        _ => panic!("unknown instruction"),
    }
}

fn clr(mem: &mut Memory) {
    for p in &mut mem.video[..] {
        *p = false;
    }
}
fn rts(cpu: &mut Cpu) {
    cpu.pc = cpu.stack.pop().expect("stack underflow!");
}
fn jump(cpu: &mut Cpu, nnn: u16) {
    cpu.pc = nnn;
}
fn call(cpu: &mut Cpu, nnn: u16) {
    cpu.stack.push(cpu.pc).expect("stack overflow!");
    cpu.pc = nnn;
}
fn ske(cpu: &mut Cpu, x: u8, nn: u8) {
    if cpu.v[x as usize] == nn {
        cpu.pc += 2;
    }
}
fn skne(cpu: &mut Cpu, x: u8, nn: u8) {
    if cpu.v[x as usize] != nn {
        cpu.pc += 2;
    }
}
fn skre(cpu: &mut Cpu, x: u8, y: u8) {
    if cpu.v[x as usize] == cpu.v[y as usize] {
        cpu.pc += 2;
    }
}
fn skrne(cpu: &mut Cpu, x: u8, y: u8) {
    if cpu.v[x as usize] != cpu.v[y as usize] {
        cpu.pc += 2;
    }
}
fn skp(cpu: &mut Cpu, mem: &Memory, x: u8) {
    if mem.keys[cpu.v[x as usize] as usize] {
        cpu.pc +=2;
    }
}
fn sknp(cpu: &mut Cpu, mem: &Memory, x: u8) {
    if !mem.keys[cpu.v[x as usize] as usize] {
        cpu.pc +=2;
    }
}
fn load(cpu: &mut Cpu, x: u8, nn: u8) {
    cpu.v[x as usize] = nn;
}
fn loadi(cpu: &mut Cpu, nnn: u16) {
    cpu.i = nnn;
}
fn jumpi(cpu: &mut Cpu, nnn: u16) {
    cpu.pc = cpu.v[0x0] as u16 + nnn;
}
fn rand(cpu: &mut Cpu, x: u8, nn: u8) {
    cpu.v[x as usize] = nn & (cpu.rand)();
}
fn add(cpu: &mut Cpu, x: u8, nn: u8) {
    cpu.v[x as usize] = cpu.v[x as usize].wrapping_add(nn);
}
fn draw(cpu: &mut Cpu, mem: &mut Memory, x: u8, y: u8, n: u8) {
    cpu.v[0xF] = 0;
    for row in 0..n {
        let data = mem.ram[(cpu.i+row as u16) as usize];
        for col in 0..8 {
            let px = (cpu.v[x as usize] as usize + (col as usize)) % 64;
            let py = (cpu.v[y as usize] as usize + (row as usize)) % 32;
            let pos = px + 64*py;
            if data & (0x80>>col) != 0 {
                if mem.video[pos] {
                    cpu.v[0xF] = 1;
                    mem.video[pos] = false;
                } else {
                    mem.video[pos] = true;
                }
            }
        }
    }
}
fn mov(cpu: &mut Cpu, x: u8, y: u8) {
    cpu.v[x as usize] = cpu.v[y as usize];
}
fn or(cpu: &mut Cpu, x: u8, y: u8) {
    cpu.v[x as usize] |= cpu.v[y as usize];
}
fn and(cpu: &mut Cpu, x: u8, y: u8) {
    cpu.v[x as usize] &= cpu.v[y as usize];
}
fn xor(cpu: &mut Cpu, x: u8, y: u8) {
    cpu.v[x as usize] ^= cpu.v[y as usize];
}
fn addr(cpu: &mut Cpu, x: u8, y: u8) {
    let (res, of) = cpu.v[x as usize].overflowing_add(cpu.v[y as usize]);
    cpu.v[x as usize] = res;
    cpu.v[0xF] = of as u8;
}
fn sub(cpu: &mut Cpu, x: u8, y: u8) {
    let (res, of) = cpu.v[x as usize].overflowing_sub(cpu.v[y as usize]);
    cpu.v[x as usize] = res;
    cpu.v[0xF] = (!of) as u8;
}
fn shr(cpu: &mut Cpu, x: u8, y: u8) {
    cpu.v[0xF] = cpu.v[x as usize] & 0x01;
    cpu.v[x as usize] >>= 1;
}
fn subn(cpu: &mut Cpu, x: u8, y: u8) {
    let (res, of) = cpu.v[y as usize].overflowing_sub(cpu.v[x as usize]);
    cpu.v[x as usize] = res;
    cpu.v[0xF] = (!of) as u8;
}
fn shl(cpu: &mut Cpu, x: u8, y: u8) {
    cpu.v[0xF] = (cpu.v[x as usize] >> 7) & 0x01;
    cpu.v[x as usize] <<= 1;
}
fn moved(cpu: &mut Cpu, x: u8) {
    cpu.v[x as usize] = cpu.dt;
}
fn keyd(cpu: &mut Cpu, mem: &Memory, x: u8) {
    if let Some(p) = mem.keys.iter().position(|&x| x) {
        cpu.v[x as usize] = p as u8;
    } else {
        cpu.pc -=2;
    }
}
fn loadd(cpu: &mut Cpu, x: u8) {
    cpu.dt = cpu.v[x as usize];
}
fn loads(cpu: &mut Cpu, x: u8) {
    cpu.st = cpu.v[x as usize];
}
fn addi(cpu: &mut Cpu, x: u8) {
    cpu.i += cpu.v[x as usize] as u16;
}
fn ldspr(cpu: &mut Cpu, x: u8) {
    cpu.i = (cpu.v[x as usize] as u16) * 5;
}
fn bcd(cpu: &Cpu, mem: &mut Memory, x: u8) {
    let mut k = cpu.v[x as usize];
    mem.ram[(cpu.i+2) as usize] = k % 10;
    k /= 10;
    mem.ram[(cpu.i+1) as usize] = k % 10;
    k /= 10;
    mem.ram[cpu.i as usize] = k % 10;
}
fn stor(cpu: &Cpu, mem: &mut Memory, x: u8) {
    for r in 0..x {
        mem.ram[(cpu.i as usize)+(r as usize)] = cpu.v[r as usize];
    }
}
fn read(cpu: &mut Cpu, mem: &Memory, x: u8) {
    for r in 0..(x+1) {
        cpu.v[r as usize] = mem.ram[(cpu.i as usize)+(r as usize)];
    }
}
