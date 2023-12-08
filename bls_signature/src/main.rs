use bls_signatures::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use bls12_381::{Bls12, G1Affine, G2Affine, Gt};
use pairing_lib::MultiMillerLoop;

fn main() {
    // generate private key
    let mut rng = ChaCha8Rng::seed_from_u64(12);
    let private_key = PrivateKey::generate(&mut rng);

    // sign
    let message = "sample_message";
    let signature = private_key.sign(message);

    // verify
    let result = verify(&signature, message, private_key.public_key());
    println!("result: {:?}", result);
}

fn verify(signature: &Signature, message: &str, pub_key: PublicKey) -> bool {
    // h(m): generate hash of message
    let hm = hash(message.as_ref());

    // convert public key and message for pairing calculation
    let pk = pub_key.as_affine();
    let h = G2Affine::from(hm).into();

    // e(pubKey, h(m)): paring calculation
    let res_pairing1 = Bls12::multi_miller_loop(&[(&pk, &h)]);

    // convert identity element and signature for pairing calculation
    let g1_neg = -G1Affine::generator();
    let sig = G2Affine::from(*signature);

    // e(-g, sig): paring calculation
    let res_pairing2 = Bls12::multi_miller_loop(&[(&g1_neg, &sig.into())]);

    // e(pubKey, h(m)) * e(-g, sig) = 1
    (res_pairing1 + res_pairing2).final_exponentiation() == Gt::identity()
}
