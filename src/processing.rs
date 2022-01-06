struct NodeParams {}

fn rev<T: std::marker::Copy>(rev: &Vec<T>) -> Vec<T> {
    let mut new_rev = rev.clone();

    let length = rev.len();
    for i in 1..length {
        new_rev[i] = rev[length - 1 - i]
    }
    new_rev.to_vec()
}

fn delay_line<T: std::marker::Copy + std::ops::AddAssign>(
    rev: &Vec<T>,
    delay_length: usize,
    repeats: u32,
) -> Vec<T> {
    let mut new_rev = rev.clone();

    let length = rev.len();

    for i in 1..length {
        let delay_i = i - delay_length;
        if i >= delay_length {
            new_rev[i] += rev[delay_i]
        }
    }

    if (repeats > 0) {
        new_rev = delay(&new_rev, delay_length, repeats - 1);
    }
    new_rev.to_vec()
}
