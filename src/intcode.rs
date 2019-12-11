use std::convert::TryFrom;

pub trait IO {
    fn input(&mut self) -> i64;
    fn output(&mut self, v: i64);
}

pub struct Intcode<'a, T>
where
    T: IO,
{
    pc: usize,
    ram: Vec<i64>,
    relative_base: i64,
    io: &'a mut T,
}

struct RawWords {
    instruction: i64,
    param1: Option<i64>,
    param2: Option<i64>,
    param3: Option<i64>,
    relative_base: i64,
}

impl RawWords {
    fn opcode(&self) -> i64 {
        self.instruction % 100
    }

    fn param(&self, mode: i64, value: i64) -> Parameter {
        match mode {
            // position mode
            0 => Parameter::Indirect {
                address: usize::try_from(value).unwrap(),
            },
            // immediate mode
            1 => Parameter::Immediate { value },
            // relative mode
            2 => self.param(0, value + self.relative_base),
            _ => unimplemented!(),
        }
    }

    fn param1(&self) -> Parameter {
        self.param((self.instruction / 100) % 10, self.param1.unwrap())
    }

    fn param2(&self) -> Parameter {
        self.param((self.instruction / 1000) % 10, self.param2.unwrap())
    }

    fn param3(&self) -> Parameter {
        self.param((self.instruction / 10000) % 10, self.param3.unwrap())
    }
}

enum Parameter {
    Indirect { address: usize },
    Immediate { value: i64 },
}

enum Instruction {
    Add {
        op1: Parameter,
        op2: Parameter,
        dest: Parameter,
    },
    Multiply {
        op1: Parameter,
        op2: Parameter,
        dest: Parameter,
    },
    Input {
        dest: Parameter,
    },
    Output {
        from: Parameter,
    },
    JumpIfTrue {
        condition: Parameter,
        target: Parameter,
    },
    JumpIfFalse {
        condition: Parameter,
        target: Parameter,
    },
    LessThan {
        op1: Parameter,
        op2: Parameter,
        dest: Parameter,
    },
    Equals {
        op1: Parameter,
        op2: Parameter,
        dest: Parameter,
    },
    RelativeBaseOffset {
        incr: Parameter,
    },
    Halt,
}

impl<'a, T> Intcode<'a, T>
where
    T: IO,
{
    pub fn new(ram: Vec<i64>, io: &'a mut T) -> Intcode<'a, T> {
        Intcode {
            pc: 0,
            ram,
            relative_base: 0,
            io,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.decode() {
                Instruction::Add { op1, op2, dest } => {
                    let op1 = self.read(op1);
                    let op2 = self.read(op2);
                    self.write(dest, op1 + op2);
                    self.pc += 4;
                }
                Instruction::Multiply { op1, op2, dest } => {
                    let op1 = self.read(op1);
                    let op2 = self.read(op2);
                    self.write(dest, op1 * op2);
                    self.pc += 4;
                }
                Instruction::Input { dest } => {
                    let value = self.io.input();
                    self.write(dest, value);
                    self.pc += 2;
                }
                Instruction::Output { from } => {
                    let value = self.read(from);
                    self.io.output(value);
                    self.pc += 2;
                }
                Instruction::JumpIfTrue { condition, target } => {
                    if self.read(condition) != 0 {
                        self.pc = usize::try_from(self.read(target)).unwrap();
                    } else {
                        self.pc += 3;
                    }
                }
                Instruction::JumpIfFalse { condition, target } => {
                    if self.read(condition) == 0 {
                        self.pc = usize::try_from(self.read(target)).unwrap();
                    } else {
                        self.pc += 3;
                    }
                }
                Instruction::LessThan { op1, op2, dest } => {
                    if self.read(op1) < self.read(op2) {
                        self.write(dest, 1);
                    } else {
                        self.write(dest, 0);
                    };
                    self.pc += 4;
                }
                Instruction::Equals { op1, op2, dest } => {
                    if self.read(op1) == self.read(op2) {
                        self.write(dest, 1);
                    } else {
                        self.write(dest, 0);
                    };
                    self.pc += 4;
                }
                Instruction::RelativeBaseOffset { incr } => {
                    let value = self.read(incr);
                    self.relative_base += value;
                    self.pc += 2;
                }
                Instruction::Halt => return,
            }
        }
    }

    fn fetch(&self) -> RawWords {
        RawWords {
            instruction: self.ram[self.pc],
            param1: self.ram.get(self.pc + 1).cloned(),
            param2: self.ram.get(self.pc + 2).cloned(),
            param3: self.ram.get(self.pc + 3).cloned(),
            relative_base: self.relative_base,
        }
    }

    fn decode(&self) -> Instruction {
        let raw = self.fetch();
        match raw.opcode() {
            1 => Instruction::Add {
                op1: raw.param1(),
                op2: raw.param2(),
                dest: raw.param3(),
            },
            2 => Instruction::Multiply {
                op1: raw.param1(),
                op2: raw.param2(),
                dest: raw.param3(),
            },
            3 => Instruction::Input { dest: raw.param1() },
            4 => Instruction::Output { from: raw.param1() },
            5 => Instruction::JumpIfTrue {
                condition: raw.param1(),
                target: raw.param2(),
            },
            6 => Instruction::JumpIfFalse {
                condition: raw.param1(),
                target: raw.param2(),
            },
            7 => Instruction::LessThan {
                op1: raw.param1(),
                op2: raw.param2(),
                dest: raw.param3(),
            },
            8 => Instruction::Equals {
                op1: raw.param1(),
                op2: raw.param2(),
                dest: raw.param3(),
            },
            9 => Instruction::RelativeBaseOffset { incr: raw.param1() },
            99 => Instruction::Halt,
            _ => unimplemented!(),
        }
    }

    fn read(&mut self, param: Parameter) -> i64 {
        match param {
            Parameter::Indirect { address } => {
                if address >= self.ram.len() {
                    self.ram.resize(2 * address, 0);
                }
                self.ram[address]
            }
            Parameter::Immediate { value } => value,
        }
    }

    fn write(&mut self, param: Parameter, value: i64) {
        match param {
            Parameter::Indirect { address } => {
                if address >= self.ram.len() {
                    self.ram.resize(2 * address, 0);
                }
                self.ram[address] = value;
            }
            Parameter::Immediate { .. } => panic!("nonsensical write"),
        }
    }
}
