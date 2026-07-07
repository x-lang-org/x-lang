fn main() {
    let max_iter = 50;
    for y in 0..40 {
        let ci = -1.25 + (y as f64) * 2.5 / 39.0;
        let mut line = String::new();
        for x in 0..40 {
            let cr = -2.0 + (x as f64) * 2.5 / 39.0;
            let mut zr = 0.0;
            let mut zi = 0.0;
            let mut is_in = true;
            for _ in 0..max_iter {
                let zr_new = zr * zr - zi * zi + cr;
                let zi_new = 2.0 * zr * zi + ci;
                zr = zr_new;
                zi = zi_new;
                if zr * zr + zi * zi > 4.0 {
                    is_in = false;
                    break;
                }
            }
            if is_in {
                line.push('*');
            } else {
                line.push(' ');
            }
        }
        println!("{}", line);
    }
}
