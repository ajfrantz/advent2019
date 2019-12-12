use itertools::Itertools;
use std::ops::AddAssign;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

impl Position {
    fn potential_energy(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Velocity {
    x: i32,
    y: i32,
    z: i32,
}

impl Velocity {
    fn kinetic_energy(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl AddAssign<Velocity> for Position {
    fn add_assign(&mut self, other: Velocity) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl AddAssign for Velocity {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Moon {
    position: Position,
    velocity: Velocity,
}

impl Moon {
    fn new(x: i32, y: i32, z: i32) -> Moon {
        Moon {
            position: Position { x, y, z },
            velocity: Velocity { x: 0, y: 0, z: 0 },
        }
    }

    fn gravity(&self, toward: Position) -> Velocity {
        Velocity {
            x: (toward.x - self.position.x).signum(),
            y: (toward.y - self.position.y).signum(),
            z: (toward.z - self.position.z).signum(),
        }
    }

    fn total_energy(&self) -> i32 {
        self.position.potential_energy() * self.velocity.kinetic_energy()
    }
}

fn step(system: &mut [Moon; 4]) {
    for (a_idx, b_idx) in (0..4).tuple_combinations() {
        let b_pos = system[b_idx].position;
        let a = &mut system[a_idx];
        a.velocity += a.gravity(b_pos);

        let a_pos = system[a_idx].position;
        let b = &mut system[b_idx];
        b.velocity += b.gravity(a_pos);
    }

    for moon in system.iter_mut() {
        moon.position += moon.velocity;
    }
}

fn x_axes(system: &[Moon; 4]) -> (i32, i32, i32, i32, i32, i32, i32, i32) {
    (
        system[0].position.x,
        system[0].velocity.x,
        system[1].position.x,
        system[1].velocity.x,
        system[2].position.x,
        system[2].velocity.x,
        system[3].position.x,
        system[3].velocity.x,
    )
}

fn y_axes(system: &[Moon; 4]) -> (i32, i32, i32, i32, i32, i32, i32, i32) {
    (
        system[0].position.y,
        system[0].velocity.y,
        system[1].position.y,
        system[1].velocity.y,
        system[2].position.y,
        system[2].velocity.y,
        system[3].position.y,
        system[3].velocity.y,
    )
}

fn z_axes(system: &[Moon; 4]) -> (i32, i32, i32, i32, i32, i32, i32, i32) {
    (
        system[0].position.z,
        system[0].velocity.z,
        system[1].position.z,
        system[1].velocity.z,
        system[2].position.z,
        system[2].velocity.z,
        system[3].position.z,
        system[3].velocity.z,
    )
}

fn main() {
    let input: [Moon; 4] = [
        Moon::new(14, 2, 8),
        Moon::new(7, 4, 10),
        Moon::new(1, 17, 16),
        Moon::new(-4, -1, 1),
    ];

    let mut system = input;
    for _ in 0..1000 {
        step(&mut system);
    }

    let answer1: i32 = system.iter().map(|m| m.total_energy()).sum();
    dbg!(answer1);

    let mut system = input;
    let mut cycle = 0;

    let mut x_repeated = false;
    let mut y_repeated = false;
    let mut z_repeated = false;

    while !x_repeated || !y_repeated || !z_repeated {
        cycle += 1;
        step(&mut system);

        if !x_repeated && x_axes(&system) == x_axes(&input) {
            println!("x repeated after {} steps", cycle);
            x_repeated = true;
        }

        if !y_repeated && y_axes(&system) == y_axes(&input) {
            println!("y repeated after {} steps", cycle);
            y_repeated = true;
        }

        if !z_repeated && z_axes(&system) == z_axes(&input) {
            println!("z repeated after {} steps", cycle);
            z_repeated = true;
        }
    }

    let answer2: i64 = 420788524631496; // by hand, via lcm(108344, 231614, 268296)
    dbg!(answer2);
}
