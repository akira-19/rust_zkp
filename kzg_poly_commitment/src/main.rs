use bls12_381::{Bls12, G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
use pairing_lib::Engine;

// f(x) = (x-3)(x-2)(x-1) = x^3 - 6x^2 + 11x - 6
// 以上の多項式の証明を行う
fn main() {
    // テキトーな値を設定
    let tau: Scalar = Scalar::from_raw([
        0x0d632cfc2c0a1cfd,
        0x0e0a8e0d0b0c0c0b,
        0x0b0c0c0b0a0d0a0c,
        0x0c0b0c0b0c0b0c0b,
    ]);

    // G1, G2の生成元を設定
    let g1: G1Projective = G1Projective::generator();
    let g2 = G2Projective::generator();

    // global parameter：G1, G2の生成元をtauでスカラー倍した値を設定
    let gp1 = (
        g1,
        g1 * tau,
        g1 * tau.pow(&[2, 0, 0, 0]),
        g1 * tau.pow(&[3, 0, 0, 0]),
    );
    let gp2 = (
        g2,
        g2 * tau,
        g2 * tau.pow(&[2, 0, 0, 0]),
        g2 * tau.pow(&[3, 0, 0, 0]),
    );

    // f(x)の各項の係数を設定
    // f(x) = (x-3)(x-2)(x-1) = x^3 - 6x^2 + 11x - 6
    // Scalar型では、little endianで設定する
    let f0 = Scalar::from_raw([6, 0, 0, 0]).neg();
    let f1 = Scalar::from_raw([11, 0, 0, 0]);
    let f2 = Scalar::from_raw([6, 0, 0, 0]).neg();
    let f3 = Scalar::from_raw([1, 0, 0, 0]);

    //
    let comf = (gp1.0 * f0) + (gp1.1 * f1) + (gp1.2 * f2) + (gp1.3 * f3);

    let u = Scalar::from_raw([5, 0, 0, 0]);
    let v = Scalar::from_raw([24, 0, 0, 0]);

    // q(x) =(f(x) - f(u))/(x-u) = x^2 - x + 6
    let q0 = Scalar::from_raw([6, 0, 0, 0]);
    let q1 = Scalar::from_raw([1, 0, 0, 0]).neg();
    let q2 = Scalar::from_raw([1, 0, 0, 0]);

    let proof = (gp2.0 * q0) + (gp2.1 * q1) + (gp2.2 * q2);

    let g_u = gp1.0 * u;
    let g_v = gp1.0 * v;

    let l1 = comf - g_v;
    let l2 = gp2.0;
    let r1 = gp1.1 - g_u;
    let r2 = proof;

    let l1_affine = G1Affine::from(l1);
    let l2_affine = G2Affine::from(l2);
    let r1_affine = G1Affine::from(r1);
    let r2_affine = G2Affine::from(r2);

    let res_pairing1 = Bls12::pairing(&l1_affine, &l2_affine);
    let res_pairing2 = Bls12::pairing(&r1_affine, &r2_affine);

    println!("res_pairing1: {:?}", res_pairing1 == res_pairing2);
}
