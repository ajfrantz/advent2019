fn meets_criteria(n: &u32) -> bool {
    let mut n = *n;
    let mut digits = [0; 6];

    for i in 0..6 {
        digits[5 - i] = n % 10;
        n /= 10;
    }

    let increasing = digits.windows(2).all(|pair| pair[0] <= pair[1]);
    if !increasing {
        return false;
    }

    let mut run_length = 1;
    let mut current = digits[0];
    for &digit in &digits[1..] {
        if digit == current {
            run_length += 1;
        } else if run_length == 2 {
            return true;
        } else {
            current = digit;
            run_length = 1;
        }
    }

    run_length == 2
}

fn main() {
    dbg!((367479..=893698).filter(meets_criteria).count());
}
