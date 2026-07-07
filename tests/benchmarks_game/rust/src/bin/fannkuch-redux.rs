fn fannkuch(n: usize) -> i32 {
    let mut perm = vec![0; n];
    let mut perm1 = vec![0; n];
    let mut count = vec![0; n];
    for i in 0..n {
        perm1[i] = i;
    }
    let mut r = n;
    let mut max_flips = 0;
    let mut checksum = 0;
    let mut perm_count = 0;
    loop {
        while r != 1 {
            count[r - 1] = r;
            r -= 1;
        }
        for i in 0..n {
            perm[i] = perm1[i];
        }
        let mut flips = 0;
        let mut k0 = perm[0];
        if k0 != 0 {
            loop {
                let mut lo = 1;
                let mut hi = k0 - 1;
                while lo < hi {
                    perm.swap(lo, hi);
                    lo += 1;
                    hi -= 1;
                }
                perm.swap(0, k0);
                flips += 1;
                k0 = perm[0];
                if k0 == 0 {
                    break;
                }
            }
            if flips > max_flips {
                max_flips = flips;
            }
        }
        if perm_count % 2 == 0 {
            checksum += flips;
        } else {
            checksum -= flips;
        }
        loop {
            if r == n {
                println!("{}", checksum);
                return max_flips;
            }
            let first = perm1[0];
            for j in 0..r {
                perm1[j] = perm1[j + 1];
            }
            perm1[r] = first;
            count[r] -= 1;
            if count[r] > 0 {
                break;
            }
            r += 1;
        }
        perm_count += 1;
    }
}

fn main() {
    let max_flips = fannkuch(7);
    println!("Pfannkuchen(7) = {}", max_flips);
}
