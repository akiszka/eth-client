use num_bigint::BigInt;

/// calculate a value mod p, while also handling negative numbers
pub fn modulo(n: &BigInt, p: &BigInt) -> BigInt {
    let mut result = n.clone() % p.clone();

    while result.lt(&BigInt::from(0)) {
        result += p.clone();
    }

    result
}

/// calculate the modular inverse of a number
pub fn mod_inverse(n: &BigInt, p: &BigInt) -> BigInt {
    if p == &BigInt::from(1) {
        return BigInt::from(1);
    }

    let (mut a, mut m, mut x, mut inv) = (n.clone(), p.clone(), BigInt::from(0), BigInt::from(1));

    while a > BigInt::from(1) {
        let div = a.clone() / m.clone();
        let rem = a.clone() % m.clone();
        inv -= div * &x;
        a = rem;
        std::mem::swap(&mut a, &mut m);
        std::mem::swap(&mut x, &mut inv);
    }

    if inv < BigInt::from(0) {
        inv += p
    }

    inv
}

pub fn legendre_symbol(a: &BigInt, p: &BigInt) -> BigInt {
    let half_p = (p.clone() - 1) / 2;
    a.modpow(&half_p, p)
}

/// Tonelli–Shanks algorithm
/// this algorithm is used to calculate the square root of a number modulo a prime number
/// I HATE THOSE GUYS WHY DID THEY HAVE TO MAKE THIS SO COMPLICATED
pub fn mod_sqrt(n: &BigInt, p: &BigInt) -> Option<BigInt> {
    if legendre_symbol(n, p) != BigInt::from(1) {
        return None;
    }

    let mut s = 0;

    while (p - 1) % BigInt::from(2).pow(s.clone()) == BigInt::from(0) {
        s += 1;
    }

    s -= 1;
    let q = (p - 1) / BigInt::from(2).pow(s);

    // Select a z such that z is a quadratic non-residue modulo p
    let mut z = BigInt::from(2);
    let mut zl = legendre_symbol(&z, p);

    while zl != p - 1 {
        z += 1;
        zl = legendre_symbol(&z, p);
    }

    let mut c = z.modpow(&q, p);
    let mut r = n.modpow(&((q.clone() + 1) / 2), p);
    let mut t = n.modpow(&q, p);
    let mut m = s;

    while modulo(&t, p) != BigInt::from(1) {
        let mut i = 1;
        let mut div = false;

        while div == false {
            i += 1;
            t = t.modpow(&BigInt::from(2), p);
            if modulo(&t, p) == BigInt::from(1) {
                div = true;
            }
        }

        let b = c.modpow(&BigInt::from(2).pow(m - i - 1), p);
        r = modulo(&(r * b.clone()), p);
        t = modulo(&(t * b.pow(2)), p);
        c = modulo(&b.pow(2), p);
        m = i;
    }

    Some(r)
}
