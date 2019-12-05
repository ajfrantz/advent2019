use std::convert::TryFrom;

struct Intcode {
    pc: usize,
    ram: Vec<i64>,
}

struct RawWords {
    instruction: i64,
    param1: Option<i64>,
    param2: Option<i64>,
    param3: Option<i64>,
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
    Halt,
}

impl Intcode {
    fn new(ram: Vec<i64>) -> Intcode {
        Intcode { pc: 0, ram }
    }

    fn run(&mut self) {
        loop {
            match self.decode() {
                Instruction::Add { op1, op2, dest } => {
                    self.write(dest, self.read(op1) + self.read(op2));
                    self.pc += 4;
                }
                Instruction::Multiply { op1, op2, dest } => {
                    self.write(dest, self.read(op1) * self.read(op2));
                    self.pc += 4;
                }
                Instruction::Input { dest } => {
                    println!("Input required.");
                    let value = || loop {
                        let mut input = String::new();
                        std::io::stdin()
                            .read_line(&mut input)
                            .expect("input required");
                        if let Ok(n) = input.trim().parse::<i64>() {
                            return n;
                        }
                        println!("Invalid integer, try again.");
                    };
                    self.write(dest, value());
                    self.pc += 2;
                }
                Instruction::Output { from } => {
                    println!("{}", self.read(from));
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
            99 => Instruction::Halt,
            _ => unimplemented!(),
        }
    }

    fn read(&self, param: Parameter) -> i64 {
        match param {
            Parameter::Indirect { address } => self.ram[address],
            Parameter::Immediate { value } => value,
        }
    }

    fn write(&mut self, param: Parameter, value: i64) {
        match param {
            Parameter::Indirect { address } => self.ram[address] = value,
            Parameter::Immediate { .. } => panic!("nonsensical write"),
        }
    }
}

fn main() {
    let ram = vec![
        3, 225, 1, 225, 6, 6, 1100, 1, 238, 225, 104, 0, 1002, 114, 46, 224, 1001, 224, -736, 224,
        4, 224, 1002, 223, 8, 223, 1001, 224, 3, 224, 1, 223, 224, 223, 1, 166, 195, 224, 1001,
        224, -137, 224, 4, 224, 102, 8, 223, 223, 101, 5, 224, 224, 1, 223, 224, 223, 1001, 169,
        83, 224, 1001, 224, -90, 224, 4, 224, 102, 8, 223, 223, 1001, 224, 2, 224, 1, 224, 223,
        223, 101, 44, 117, 224, 101, -131, 224, 224, 4, 224, 1002, 223, 8, 223, 101, 5, 224, 224,
        1, 224, 223, 223, 1101, 80, 17, 225, 1101, 56, 51, 225, 1101, 78, 89, 225, 1102, 48, 16,
        225, 1101, 87, 78, 225, 1102, 34, 33, 224, 101, -1122, 224, 224, 4, 224, 1002, 223, 8, 223,
        101, 7, 224, 224, 1, 223, 224, 223, 1101, 66, 53, 224, 101, -119, 224, 224, 4, 224, 102, 8,
        223, 223, 1001, 224, 5, 224, 1, 223, 224, 223, 1102, 51, 49, 225, 1101, 7, 15, 225, 2, 110,
        106, 224, 1001, 224, -4539, 224, 4, 224, 102, 8, 223, 223, 101, 3, 224, 224, 1, 223, 224,
        223, 1102, 88, 78, 225, 102, 78, 101, 224, 101, -6240, 224, 224, 4, 224, 1002, 223, 8, 223,
        101, 5, 224, 224, 1, 224, 223, 223, 4, 223, 99, 0, 0, 0, 677, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1105, 0, 99999, 1105, 227, 247, 1105, 1, 99999, 1005, 227, 99999, 1005, 0, 256, 1105, 1,
        99999, 1106, 227, 99999, 1106, 0, 265, 1105, 1, 99999, 1006, 0, 99999, 1006, 227, 274,
        1105, 1, 99999, 1105, 1, 280, 1105, 1, 99999, 1, 225, 225, 225, 1101, 294, 0, 0, 105, 1, 0,
        1105, 1, 99999, 1106, 0, 300, 1105, 1, 99999, 1, 225, 225, 225, 1101, 314, 0, 0, 106, 0, 0,
        1105, 1, 99999, 1107, 226, 677, 224, 102, 2, 223, 223, 1006, 224, 329, 101, 1, 223, 223,
        1108, 226, 677, 224, 1002, 223, 2, 223, 1005, 224, 344, 101, 1, 223, 223, 8, 226, 677, 224,
        102, 2, 223, 223, 1006, 224, 359, 1001, 223, 1, 223, 1007, 226, 677, 224, 1002, 223, 2,
        223, 1005, 224, 374, 101, 1, 223, 223, 1008, 677, 677, 224, 1002, 223, 2, 223, 1005, 224,
        389, 1001, 223, 1, 223, 1108, 677, 226, 224, 1002, 223, 2, 223, 1006, 224, 404, 1001, 223,
        1, 223, 1007, 226, 226, 224, 1002, 223, 2, 223, 1005, 224, 419, 1001, 223, 1, 223, 1107,
        677, 226, 224, 1002, 223, 2, 223, 1006, 224, 434, 101, 1, 223, 223, 108, 677, 677, 224,
        1002, 223, 2, 223, 1005, 224, 449, 1001, 223, 1, 223, 1107, 677, 677, 224, 102, 2, 223,
        223, 1005, 224, 464, 1001, 223, 1, 223, 108, 226, 226, 224, 1002, 223, 2, 223, 1006, 224,
        479, 1001, 223, 1, 223, 1008, 226, 226, 224, 102, 2, 223, 223, 1005, 224, 494, 101, 1, 223,
        223, 108, 677, 226, 224, 102, 2, 223, 223, 1005, 224, 509, 1001, 223, 1, 223, 8, 677, 226,
        224, 1002, 223, 2, 223, 1006, 224, 524, 101, 1, 223, 223, 7, 226, 677, 224, 1002, 223, 2,
        223, 1006, 224, 539, 101, 1, 223, 223, 7, 677, 226, 224, 102, 2, 223, 223, 1006, 224, 554,
        1001, 223, 1, 223, 7, 226, 226, 224, 1002, 223, 2, 223, 1006, 224, 569, 101, 1, 223, 223,
        107, 677, 677, 224, 102, 2, 223, 223, 1006, 224, 584, 101, 1, 223, 223, 1108, 677, 677,
        224, 102, 2, 223, 223, 1006, 224, 599, 1001, 223, 1, 223, 1008, 677, 226, 224, 1002, 223,
        2, 223, 1005, 224, 614, 1001, 223, 1, 223, 8, 677, 677, 224, 1002, 223, 2, 223, 1006, 224,
        629, 1001, 223, 1, 223, 107, 226, 677, 224, 1002, 223, 2, 223, 1006, 224, 644, 101, 1, 223,
        223, 1007, 677, 677, 224, 102, 2, 223, 223, 1006, 224, 659, 101, 1, 223, 223, 107, 226,
        226, 224, 1002, 223, 2, 223, 1006, 224, 674, 1001, 223, 1, 223, 4, 223, 99, 226,
    ];

    Intcode::new(ram).run();
}
