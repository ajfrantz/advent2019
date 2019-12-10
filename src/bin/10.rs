use itertools::{iproduct, Itertools};
use num::Integer;
use std::convert::TryFrom;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Cell {
    Empty,
    Asteroid,
}

impl Cell {
    fn parse(c: char) -> Cell {
        match c {
            '.' => Cell::Empty,
            '#' => Cell::Asteroid,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Position(i64, i64);

impl Position {
    fn new(raw: (i64, i64)) -> Position {
        Position(raw.0, raw.1)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Ray(i64, i64);

impl Ray {
    fn new(raw: (i64, i64)) -> Ray {
        Ray(raw.0, raw.1).reduce()
    }

    fn reduce(&self) -> Ray {
        let gcd = self.0.gcd(&self.1);
        let x = self.0 / gcd;
        let y = self.1 / gcd;
        if x == 0 {
            Ray(0, y / y.abs())
        } else if y == 0 {
            Ray(x / x.abs(), 0)
        } else {
            Ray(x, y)
        }
    }

    fn angle(&self) -> f64 {
        let y = self.1 as f64;
        let x = self.0 as f64;
        // This is a strange formulation:
        //  - x.atan2(y) rotates us so that the -pi/pi boundary point is "up"
        //  - negating the whole thing gives us the clockwise rotation we need
        -(x.atan2(y))
    }
}

impl Add<Ray> for Position {
    type Output = Self;

    fn add(self, other: Ray) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for Position {
    type Output = Ray;

    fn sub(self, other: Position) -> Ray {
        Ray::new((self.0 - other.0, self.1 - other.1))
    }
}

#[derive(Debug)]
struct Map {
    rows: Vec<Vec<Cell>>,
    width: i64,
    height: i64,
}

impl Map {
    fn new(input: &str) -> Map {
        let rows: Vec<Vec<Cell>> = input
            .split('\n')
            .map(|row| row.chars().map(|c| Cell::parse(c)).collect())
            .collect();
        let height = rows.len() as i64;
        let width = rows[0].len() as i64;

        Map {
            rows,
            width,
            height,
        }
    }

    fn cell(&self, position: Position) -> Option<Cell> {
        let x = usize::try_from(position.0).ok()?;
        let y = usize::try_from(position.1).ok()?;
        self.rows.get(y)?.get(x).cloned()
    }

    fn cell_mut(&mut self, position: Position) -> Option<&mut Cell> {
        let x = usize::try_from(position.0).ok()?;
        let y = usize::try_from(position.1).ok()?;
        self.rows.get_mut(y)?.get_mut(x)
    }

    fn asteroids(&self) -> impl Iterator<Item = Position> + '_ {
        iproduct!(0..self.width, 0..self.height)
            .map(Position::new)
            .filter(move |&p| self.cell(p) == Some(Cell::Asteroid))
    }

    fn visible_from(&self, origin: Position) -> impl Iterator<Item = Ray> + '_ {
        self.asteroids()
            .filter(move |&p| p != origin)
            .map(move |p| p - origin)
            .unique()
    }

    fn fire_laser(&mut self, from: Position, direction: Ray) -> Option<Position> {
        let mut position = from + direction;
        loop {
            match self.cell_mut(position) {
                None => return None,
                Some(Cell::Empty) => (),
                Some(c) => {
                    *c = Cell::Empty;
                    return Some(position);
                }
            }

            position = position + direction;
        }
    }
}

fn main() {
    let mut map = Map::new(MAP);

    let answer1 = map
        .asteroids()
        .map(|base| (map.visible_from(base).count(), base))
        .max_by_key(|t| t.0)
        .unwrap();
    dbg!(answer1);

    // This repeats some work already done above but... w/e.
    let (_, base) = answer1;
    let mut shot_number = 1;
    loop {
        let mut shots: Vec<Ray> = map.visible_from(base).collect();
        shots.sort_by(|a, b| a.angle().partial_cmp(&b.angle()).unwrap());
        if shots.is_empty() {
            break;
        }

        for direction in shots {
            let vaporized = map.fire_laser(base, direction).expect("blew something up");
            if shot_number == 200 {
                let answer2 = vaporized.0 * 100 + vaporized.1;
                dbg!(answer2);
                return;
            }

            shot_number += 1;
        }
    }

    println!("fired {} shots", shot_number - 1);
}

const MAP: &str = ".###.###.###.#####.#
#####.##.###..###..#
.#...####.###.######
######.###.####.####
#####..###..########
#.##.###########.#.#
##.###.######..#.#.#
.#.##.###.#.####.###
##..#.#.##.#########
###.#######.###..##.
###.###.##.##..####.
.##.####.##########.
#######.##.###.#####
#####.##..####.#####
##.#.#####.##.#.#..#
###########.#######.
#.##..#####.#####..#
#####..#####.###.###
####.#.############.
####.#.#.##########.";
