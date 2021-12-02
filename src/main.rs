struct Cpu {
    memory: Box<[u8]>,
    r: Box<[u16]>,
    pc: u16,
    ir: u32,
    a: u16,
    b: u16,
    n: bool,
    z: bool,
    v: bool,
    c: bool,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            memory: vec![0; 65536].into_boxed_slice(),
            r: vec![0; 32].into_boxed_slice(),
            pc: 0,
            ir: 0,
            a: 0,
            b: 0,
            n: false,
            z: false,
            v: false,
            c: false,
        }
    }

    fn custom() -> Self {
        let mut cpu = Cpu::new();
        cpu.pc = 0x1000;
        cpu.a = 0x007C;
        cpu.r[1] = 0x0006;
        cpu.r[2] = 0x0030;
        cpu.r[3] = 0x8765;
        cpu.r[4] = 0x0007;
        cpu.memory[0x0000] = 0x0000;
        cpu.memory[0x0001] = 0x000A;
        cpu.memory[0x0002] = 0x0000;
        cpu.memory[0x0003] = 0x0000;
        cpu.memory[0x0004] = 0x0000;
        cpu.memory[0x0005] = 0x000B;
        cpu.memory[0x0006] = 0x00FF;
        cpu.memory[0x0007] = 0x0056;
        cpu.memory[0x0008] = 0x004C;
        cpu.memory[0x0009] = 0x00B8;
        cpu.memory[0x000A] = 0x0000;
        cpu.memory[0x000B] = 0x0080;
        cpu.memory[0x000C] = 0x0041;
        cpu.memory[0x000D] = 0x0000;
        cpu.memory[0x000E] = 0x00E3;
        cpu.memory[0x000F] = 0x0013;
        cpu.memory[0x0010] = 0x0000;
        cpu.memory[0x0011] = 0x0000;
        cpu.memory[0x0012] = 0x0029;
        cpu.memory[0x0013] = 0x0005;
        cpu.memory[0x1000] = 0x00C0;
        cpu.memory[0x1001] = 0x00C0;
        cpu.memory[0x1002] = 0x0000;
        cpu.memory[0x1003] = 0x0010;
        cpu.memory[0x1004] = 0x00C3;
        cpu.memory[0x1005] = 0x00E0;
        cpu.memory[0x1006] = 0x0000;
        cpu.memory[0x1007] = 0x0008;
        cpu.memory[0x1008] = 0x0000;
        cpu.memory[0x1009] = 0x0013;
        cpu.memory[0x100A] = 0x00C0;
        cpu.memory[0x100B] = 0x0061;
        cpu.memory[0x100C] = 0x00C1;
        cpu.memory[0x100D] = 0x0062;
        cpu.memory[0x100E] = 0x00C0;
        cpu.memory[0x100F] = 0x00A0;
        cpu.memory[0x1010] = 0x0000;
        cpu.memory[0x1011] = 0x0010;
        cpu.memory[0x1012] = 0x00C2;
        cpu.memory[0x1013] = 0x00E0;
        cpu.memory[0x1014] = 0x0000;
        cpu.memory[0x1015] = 0x0002;
        cpu.memory[0x1016] = 0x00C1;
        cpu.memory[0x1017] = 0x00A0;
        cpu.memory[0x1018] = 0x0000;
        cpu.memory[0x1019] = 0x0010;
        cpu.memory[0x101A] = 0x0040;
        cpu.memory[0x101B] = 0x0010;
        cpu.memory[0x101C] = 0x0000;
        cpu.memory[0x101D] = 0x00C4;
        cpu.memory[0x101E] = 0x0003;
        cpu.memory[0x101F] = 0x00C1;
        cpu.memory[0x1020] = 0x0084;
        cpu.memory[0x1021] = 0x00FF;
        cpu
    }

    fn read_u8(&self, address: u16) -> u8 {
        let value = self.memory[address as usize];
        log_read(address, value);
        value
    }

    fn read_u16(&self, address: u16) -> u16 {
        let value_first_half = self.memory[address as usize];
        log_read(address, value_first_half);
        let value_second_half = self.memory[(address + 1) as usize];
        log_read(address + 1, value_second_half);
        let value = (value_first_half as u16) << 8 | value_second_half as u16;
        value
    }

    fn write_u16(&mut self, address: u16, value: u16) {
        self.memory[address as usize] = (value >> 8) as u8;
        log_write(address, (value >> 8) as u8);
        self.memory[(address + 1) as usize] = value as u8;
        log_write(address + 1, value as u8);
    }

    fn fetch(&mut self) {
        println!("\n== FETCHING (PC = {:04X})\n", self.pc);

        let opcode = self.read_u8(self.pc);

        match opcode {
            0b00000000 => self.fetch_bz(opcode),
            0b01000000 | 0b01000001 => self.fetch_jump(opcode),
            0b10000000 | 0b10000001 |
            0b10000010 | 0b10000011 => self.fetch_addressless(opcode),
            0b11000000 | 0b11000001 |
            0b11000010 | 0b11000011 |
            0b11000100 | 0b11000101 => self.fetch_addressfull(opcode),
            _ => panic!("Read invalid opcode {:b}", opcode),
        }

        println!("PC = {:X}", self.pc);
    }

    // BZ
    fn fetch_bz(&mut self, opcode: u8) {
        let offset = self.read_u8(self.pc + 1);
        self.ir = (opcode as u32) << 24 | (offset as u32) << 16;
        self.pc += 2;
    }

    // JMP, JSR
    fn fetch_jump(&mut self, opcode: u8) {
        let address = self.read_u16(self.pc + 1);
        self.ir = (opcode as u32) << 24 | (address as u32) << 8;
        self.pc += 3;
    }

    // PUSH, POP, RTS, RTI
    fn fetch_addressless(&mut self, opcode: u8) {
        self.ir = (opcode as u32) << 24;
        self.pc += 1;
    }

    fn fetch_addressfull(&mut self, opcode: u8) {
        let addr_mode = self.read_u8(self.pc + 1);
        let bit7 = addr_mode >> 7;

        if bit7 == 0 {
            // uses R registers
            self.ir = (opcode as u32) << 24 | (addr_mode as u32) << 16;
            self.pc += 2;
        } else {
            // has immediate address
            let addressing = addr_mode >> 5;
            // memory direct or memory indirect
            if addressing == 0b101 || addressing == 0b110 || addressing == 0b111 {
                let address = self.read_u16(self.pc + 2);
                self.ir = (opcode as u32) << 24 | (addr_mode as u32) << 16 | address as u32;
                self.pc += 4;
            } else if addressing == 0b100 {
                let offset = self.read_u8(self.pc + 2);
                self.ir = (opcode as u32) << 24 | (addr_mode as u32) << 16 | (offset as u32) << 8;
                self.pc += 3;
            } else {
                panic!("Read invalid addressing mode: {}", addr_mode);
            }
        }
    }

    fn decode(&mut self) {
        println!("\n== DECODING\n");

        let opcode = (self.ir >> 24) as u8;
        let addr_mode = (self.ir >> 16) as u8;
        let addressing = addr_mode >> 5;

        match opcode {
            // addressfull instructions
            0b11000000 | 0b11000001 |
            0b11000010 | 0b11000011 |
            0b11000100 | 0b11000101 => {
                match addressing {
                    0b110 => self.memory_indirect(),
                    0b111 => self.immediate(),
                    0b011 => self.reg_indirect_preincrement(),
                    0b101 => self.memory_direct(),
                    0b000 => self.reg_direct(),
                    0b100 => self.reg_indirect_offset(),
                    _ => panic!("Invalid addressing mode ({:b}) while decoding", addressing),
                }
            },
            _ => println!("Skipping decode"),
        }
    }

    fn memory_indirect(&mut self) {
        println!("Memory indirect addressing");
        let address_of_address = self.ir as u16;

        println!("Reading address");
        let address = self.read_u16(address_of_address);

        // TODO: maybe move this up or down
        // check for ST
        if self.ir >> 24 == 0b11000001 {
            return;
        }

        println!("Reading operand");
        let operand = self.read_u16(address);

        println!("Operand: {:04X}", operand);
        self.b = operand;
    }

    fn immediate(&mut self) {
        println!("Immediate addressing");
        let operand = self.ir as u16; // TODO: check if immediate is always rightmost bytes
        println!("Operand: {:04X}", operand);
        self.b = operand;
    }

    fn reg_indirect_preincrement(&mut self) {
        println!("Register indirect with preincrement addressing");
        let reg_idx = ((self.ir >> 16) & 0b11111) as usize; // which of r registers to use
        self.r[reg_idx] += 2; // operand is 2 bytes long

        // check for ST
        if self.ir >> 24 == 0b11000001 {
            return;
        }

        println!("Reading operand");
        let operand = self.read_u16(self.r[reg_idx]);

        println!("Operand: {:04X}", operand);
        self.b = operand; // TODO: remove mofifying b for st

        println!("Changed R{} = {:04X}", reg_idx, self.r[reg_idx]);
    }

    fn memory_direct(&mut self) {
        println!("Memory direct addressing");

        // check for ST
        if self.ir >> 24 == 0b11000001 {
            return;
        }

        let address = self.ir as u16; // TODO: check if memdirect is always rightmost bytes
        println!("Address is {:04X}", address);

        println!("Reading operand");
        let operand = self.read_u16(address);

        println!("Operand: {:04X}", operand);
        self.b = operand;
    }

    fn reg_direct(&mut self) {
        println!("Register direct addressing");
        let reg_idx = ((self.ir >> 16) & 0b11111) as usize; // which of r registers to use
        let operand = self.r[reg_idx];
        println!("Operand: {:04X}", operand);
        self.b = operand;
    }

    fn reg_indirect_offset(&mut self) {
        println!("Register indirect with offset addressing");

        // check for ST
        if self.ir >> 24 == 0b11000001 {
            return;
        }

        let reg_idx = ((self.ir >> 16) & 0b11111) as usize; // which of r registers to use
        let offset = ((self.ir >> 8) as i8) as i16;

        let operand_address = (self.r[reg_idx] as i16 + offset) as u16;
        let operand = self.read_u16(operand_address);

        println!("Operand: {:04X}", operand);
        self.b = operand;
    }

    fn execute(&mut self) {
        println!("\n== EXECUTING\n");

        let opcode = (self.ir >> 24) as u8;
        match opcode {
            0b11000000 => self.exec_ld(),
            0b11000001 => self.exec_st(),
            0b11000011 => self.exec_and(),
            0b00000000 => self.exec_bz(),
            0b11000010 => self.exec_add(),
            0b01000000 => self.exec_jmp(),
            0b11000100 => self.exec_asr(),
            _ => panic!("Kukuuuuuu"),
        }

        println!("A = {:04X}, N = {}, Z = {}, V = {}, C = {}", self.a, self.n, self.z, self.v, self.c);
    }

    fn exec_ld(&mut self) {
        self.a = self.b;

        // negative
        self.n = self.a >> 15 == 1;

        // zero
        self.z = self.a == 0;

        println!("Executed LD");
    }

    fn exec_st(&mut self) {
        let addr_mode = (self.ir >> 16) as u8;
        let addressing = addr_mode >> 5;
        match addressing {
            // register indirect with preincrement
            0b011 => {
                let reg_idx = ((self.ir >> 16) & 0b11111) as usize; // which of r registers to use
                self.write_u16(self.r[reg_idx], self.a);
            }

            // memory direct
            0b101 => {
                let address = self.ir as u16;
                self.write_u16(address, self.a);
            }

            // register indirect with offset
            0b100 => {
                let reg_idx = ((self.ir >> 16) & 0b11111) as usize; // which of r registers to use
                let offset = ((self.ir >> 8) as i8) as i16;
                let operand_address = (self.r[reg_idx] as i16 + offset) as u16;

                self.write_u16(operand_address, self.a);
            }

            _ => (),
        }
        // TODO: implement writing to registers

        println!("Executed ST");
    }

    fn exec_and(&mut self) {
        self.a = self.a & self.b;

        // negative
        self.n = self.a >> 15 == 1;

        // zero
        self.z = self.a == 0;

        println!("Executed AND");
    }

    fn exec_bz(&mut self) {
        if self.z {
            let offset = (self.ir >> 16) as i8;
            if offset < 0 {
                let offset = offset * -1;
                self.pc = self.pc.wrapping_sub(offset as u16);
            } else {
                self.pc = self.pc.wrapping_add(offset as u16);
            }
            println!("offset = {} ({:04X})", offset, offset);
            println!("PC = {:04X}", self.pc);
        }

        println!("Executed BZ");
    }

    fn exec_add(&mut self) {
        let a = self.a as i16;
        let b = self.b as i16;
        let (sum, overflow) = a.overflowing_add(b);

        self.a = sum as u16;

        // negative
        self.n = self.a >> 15 == 1;

        // zero
        self.z = self.a == 0;

        // overflow
        self.v = overflow;

        // carry
        self.c = overflow;

        println!("Executed ADD");
    }

    fn exec_jmp(&mut self) {
        let address = (self.ir >> 8) as u16;
        self.pc = address;
        println!("PC = {:04X}", address);

        println!("Executed JMP");
    }

    fn exec_asr(&mut self) {
        let mut operand = self.b as i16;
        let carry = operand & 1;

        operand >>= 1;
        self.a = operand as u16;

        // negative
        self.n = self.a >> 15 == 1;

        // zero
        self.z = self.a == 0;

        // carry
        self.c = carry == 1;

        println!("Executed ASR");
    }
}

fn log_read(address: u16, value: u8) {
    println!("read from {:04X} value {:02X} ({:08b})", address, value, value);
}

fn log_write(address: u16, value: u8) {
    println!("wrote to {:04X} value {:02X} ({:08b})", address, value, value);
}

fn main() {
    let mut cpu = Cpu::custom();
    for i in 1..=14 {
        println!("\n=== Instruction {}", i);
        cpu.fetch();
        cpu.decode();
        cpu.execute();
    }
}
