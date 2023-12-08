use bls12_381::{Bls12, G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use pairing_lib::Engine;

fn main() {
    // pick random tau
    let tau: Scalar = Scalar::from_raw([
        0x0d632cfc2c0a1cfd,
        0x0e0a8e0d0b0c0c0b,
        0x0b0c0c0b0a0d0a0c,
        0x0c0b0c0b0c0b0c0b,
    ]);

    // example
    // f(x) = (x-3)(x-2)(x-1) = x^3 - 6x^2 + 11x - 6
    // f0 = -6, f1 = 11, f2 = -6, f3 = 1
    // user little endian in Scalar type
    let f0 = Scalar::from_raw([6, 0, 0, 0]).neg();
    let f1 = Scalar::from_raw([11, 0, 0, 0]);
    let f2 = Scalar::from_raw([6, 0, 0, 0]).neg();
    let f3 = Scalar::from_raw([1, 0, 0, 0]);

    let f = Vec::from([f0, f1, f2, f3]);

    // generate global parameters
    // gp1 = [g^tau^0, g^tau^1, g^tau^2, g^tau^3]
    // gp2 = [g^tau^0, g^tau^1, g^tau^2, g^tau^3]
    let (gp1, gp2) = generate_gp(f.len() as u16, tau);

    // calculate commitment f
    let comf = calc_comf(gp1.clone(), f.clone());

    // pick random u (this is done by the verifier)
    let u = Scalar::from_raw([5, 0, 0, 0]);
    let v = calc_f(f.clone(), u);

    // calculate q(x)
    let q = calc_q(f.clone(), u, v);

    // calculate proof
    let proof = calc_proof(gp2.clone(), q.clone());

    // verify
    // e(commitment / g^v, g) == e(g^(tau-u), proof)
    // commitment = comf
    // g^v = gp1[0] * v
    // g = gp2[0]
    // g^(tau-u) = gp1[1] - gp1[0] * u
    // proof = proof

    let g_u = gp1[0] * u;
    let g_v = gp1[0] * v;

    // calculate left side of the pairing equation
    let l1 = comf - g_v;
    let l2 = gp2[0];

    // calculate right side of the pairing equation
    let r1 = gp1[1] - g_u;
    let r2 = proof;

    // convert to affine
    let l1_affine = G1Affine::from(l1);
    let l2_affine = G2Affine::from(l2);
    let r1_affine = G1Affine::from(r1);
    let r2_affine = G2Affine::from(r2);

    // calculate pairing
    let res_pairing1 = Bls12::pairing(&l1_affine, &l2_affine);
    let res_pairing2 = Bls12::pairing(&r1_affine, &r2_affine);

    // compare result of the pairing
    println!("res_pairing1: {:?}", res_pairing1 == res_pairing2);
}

fn generate_gp(degree: u16, tau: Scalar) -> (Vec<G1Projective>, Vec<G2Projective>) {
    let g1 = G1Projective::generator();
    let g2 = G2Projective::generator();

    let mut gp1: Vec<G1Projective> = Vec::new();
    let mut gp2: Vec<G2Projective> = Vec::new();

    for i in 0..degree {
        gp1.push(g1 * tau.pow(&[i as u64, 0, 0, 0]));
        gp2.push(g2 * tau.pow(&[i as u64, 0, 0, 0]));
    }

    (gp1, gp2)
}

fn calc_comf(gp1: Vec<G1Projective>, f: Vec<Scalar>) -> G1Projective {
    gp1.iter()
        .zip(f.iter())
        .map(|(&x, &y)| x * y)
        .fold(G1Projective::identity(), |acc, e: G1Projective| acc + e)
}

fn calc_proof(gp2: Vec<G2Projective>, q: Vec<Scalar>) -> G2Projective {
    gp2.iter()
        .zip(q.iter())
        .map(|(&x, &y)| x * y)
        .fold(G2Projective::identity(), |acc, e: G2Projective| acc + e)
}

fn calc_f(f: Vec<Scalar>, u: Scalar) -> Scalar {
    f.iter()
        .zip(0..)
        .map(|(&x, y)| x * u.pow(&[y as u64, 0, 0, 0]))
        .fold(Scalar::zero(), |acc, e| acc + e)
}

fn calc_q(f: Vec<Scalar>, u: Scalar, v: Scalar) -> Vec<Scalar> {
    let mut dividend = f.clone();
    dividend[0] = dividend[0] - v;

    let divider = Vec::from([u.neg(), Scalar::from_raw([1, 0, 0, 0])]);

    let mut quotient = Vec::new();

    while dividend.len() >= divider.len() {
        let scale = dividend.pop().unwrap();
        quotient.insert(0, scale);
        let last_index = dividend.len() - 1;

        dividend[last_index] = dividend[last_index] + (u * scale);
    }

    return quotient;
}
