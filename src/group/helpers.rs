use group::Group;

pub fn scalar_mul_i64<G: Group>(g: G, scalar: i64) -> G {
    let s = G::Scalar::from(scalar.unsigned_abs());
    if scalar < 0 { -g * s } else { g * s }
}

pub fn generator_scalar_mul_i64<G: Group>(scalar: i64) -> G {
    let s = G::Scalar::from(scalar.unsigned_abs());
    if scalar < 0 {
        -G::mul_by_generator(&s)
    } else {
        G::mul_by_generator(&s)
    }
}
