use itertools::Itertools;
use std::convert::TryFrom;
use std::sync::mpsc::channel;

struct Intcode<I, O>
where
    I: FnMut() -> i64,
    O: FnMut(i64),
{
    pc: usize,
    ram: Vec<i64>,
    input: I,
    output: O,
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

impl<I, O> Intcode<I, O>
where
    I: FnMut() -> i64,
    O: FnMut(i64),
{
    fn new(ram: Vec<i64>, input: I, output: O) -> Intcode<I, O> {
        Intcode {
            pc: 0,
            ram,
            input,
            output,
        }
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
                    let value = (self.input)();
                    self.write(dest, value);
                    self.pc += 2;
                }
                Instruction::Output { from } => {
                    let value = self.read(from);
                    (self.output)(value);
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
        3, 8, 1001, 8, 10, 8, 105, 1, 0, 0, 21, 38, 47, 64, 89, 110, 191, 272, 353, 434, 99999, 3,
        9, 101, 4, 9, 9, 102, 3, 9, 9, 101, 5, 9, 9, 4, 9, 99, 3, 9, 1002, 9, 5, 9, 4, 9, 99, 3, 9,
        101, 2, 9, 9, 102, 5, 9, 9, 1001, 9, 5, 9, 4, 9, 99, 3, 9, 1001, 9, 5, 9, 102, 4, 9, 9,
        1001, 9, 5, 9, 1002, 9, 2, 9, 1001, 9, 3, 9, 4, 9, 99, 3, 9, 102, 2, 9, 9, 101, 4, 9, 9,
        1002, 9, 4, 9, 1001, 9, 4, 9, 4, 9, 99, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9,
        3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 101,
        1, 9, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4,
        9, 3, 9, 101, 2, 9, 9, 4, 9, 99, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9,
        102, 2, 9, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 2, 9,
        9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3,
        9, 102, 2, 9, 9, 4, 9, 99, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 101,
        1, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 1002, 9, 2, 9,
        4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9,
        101, 1, 9, 9, 4, 9, 99, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 1001, 9,
        1, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9,
        3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 1, 9, 4, 9, 3, 9, 1002,
        9, 2, 9, 4, 9, 99, 3, 9, 101, 1, 9, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 1001, 9, 2, 9,
        4, 9, 3, 9, 1001, 9, 2, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9, 102, 2, 9, 9, 4, 9, 3, 9,
        1001, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9, 2, 9, 4, 9, 3, 9, 1002, 9,
        2, 9, 4, 9, 99,
    ];

    let answer1 = (0..=4)
        .permutations(5)
        .map(|phases| {
            let mut signal = 0;

            for phase in &phases {
                let returns = [*phase, signal];
                let mut calls = 0;
                Intcode::new(
                    ram.clone(),
                    move || {
                        calls += 1;
                        returns[calls - 1]
                    },
                    |v| {
                        signal = v;
                    },
                )
                .run();
            }

            (signal, phases)
        })
        .max();
    dbg!(answer1);

    let answer2 = (5..=9)
        .permutations(5)
        .map(|phases| {
            // I could for-loop or macroify this, but keeping it explicit for now.
            let (input, a_in) = channel();
            let (a_out, b_in) = channel();
            let (b_out, c_in) = channel();
            let (c_out, d_in) = channel();
            let (d_out, e_in) = channel();
            let (e_out, thrusters) = channel();

            let ram_e = ram.clone();
            std::thread::spawn(move || {
                Intcode::new(
                    ram_e,
                    move || e_in.recv().unwrap(),
                    move |v| e_out.send(v).unwrap(),
                )
                .run();
            });
            d_out.send(phases[4]).expect("E must accept phase");

            let ram_d = ram.clone();
            std::thread::spawn(move || {
                Intcode::new(
                    ram_d,
                    move || d_in.recv().unwrap(),
                    move |v| d_out.send(v).unwrap(),
                )
                .run();
            });
            c_out.send(phases[3]).expect("D must accept phase");

            let ram_c = ram.clone();
            std::thread::spawn(move || {
                Intcode::new(
                    ram_c,
                    move || c_in.recv().unwrap(),
                    move |v| c_out.send(v).unwrap(),
                )
                .run();
            });
            b_out.send(phases[2]).expect("C must accept phase");

            let ram_b = ram.clone();
            std::thread::spawn(move || {
                Intcode::new(
                    ram_b,
                    move || b_in.recv().unwrap(),
                    move |v| b_out.send(v).unwrap(),
                )
                .run();
            });
            a_out.send(phases[1]).expect("B must accept phase");

            let ram_a = ram.clone();
            std::thread::spawn(move || {
                Intcode::new(
                    ram_a,
                    move || a_in.recv().unwrap(),
                    move |v| a_out.send(v).unwrap(),
                )
                .run();
            });
            input.send(phases[0]).expect("A must accept phase");

            input.send(0).expect("A must accept an input signal");

            let mut signal = 0;
            while let Ok(thruster) = thrusters.recv() {
                signal = thruster;
                let _ = input.send(thruster);
            }
            (signal, phases)
        })
        .max();
    dbg!(answer2);
}
