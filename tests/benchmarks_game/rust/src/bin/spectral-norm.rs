fn eval_a(i: usize, j: usize) -> f64 {
    1.0 / (((i + j) * (i + j + 1) / 2 + i + 1) as f64)
}

fn eval_a_times_u(u: &[f64], au: &mut [f64]) {
    let n = u.len();
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            sum += eval_a(i, j) * u[j];
        }
        au[i] = sum;
    }
}

fn eval_at_times_u(u: &[f64], atu: &mut [f64]) {
    let n = u.len();
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..n {
            sum += eval_a(j, i) * u[j];
        }
        atu[i] = sum;
    }
}

fn eval_ata_times_u(u: &[f64], atau: &mut [f64], temp: &mut [f64]) {
    eval_a_times_u(u, temp);
    eval_at_times_u(temp, atau);
}

fn main() {
    let n = 100;
    let mut u = vec![1.0; n];
    let mut v = vec![0.0; n];
    let mut temp = vec![0.0; n];
    for _ in 0..10 {
        eval_ata_times_u(&u, &mut v, &mut temp);
        eval_ata_times_u(&v, &mut u, &mut temp);
    }
    let mut v_b_v = 0.0;
    let mut v_v = 0.0;
    for i in 0..n {
        v_b_v += u[i] * v[i];
        v_v += v[i] * v[i];
    }
    let norm = (v_b_v / v_v).sqrt();
    println!("{:.9}", norm);
}
