use bls_signatures::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

use bls12_381::{Bls12, G1Affine, G2Affine, Gt};
use pairing_lib::MultiMillerLoop;

fn main() {
    // 秘密鍵の生成
    let mut rng = ChaCha8Rng::seed_from_u64(12);
    let private_key = PrivateKey::generate(&mut rng);

    // 署名
    let message = "sample_message";
    let signature = private_key.sign(message);

    // 検証
    let result = verify(&signature, message, private_key.public_key());
    println!("result: {:?}", result);
}

fn verify(signature: &Signature, message: &str, pub_key: PublicKey) -> bool {
    // h(m): メッセージのハッシュ値の生成
    let hm = hash(message.as_ref());

    // ペアリング計算用に公開鍵とメッセージの変換
    let pk = pub_key.as_affine();
    let h = G2Affine::from(hm).into();

    // e(pubKey, h(m))ペアリング計算
    let res_pairing1 = Bls12::multi_miller_loop(&[(&pk, &h)]);

    // ペアリング計算用に単位元と署名の変換
    let g1_neg = -G1Affine::generator();
    let sig = G2Affine::from(*signature);

    // e(-g, sig)ペアリング計算
    let res_pairing2 = Bls12::multi_miller_loop(&[(&g1_neg, &sig.into())]);

    // e(pubKey, h(m)) * e(-g, sig) = 1
    (res_pairing1 + res_pairing2).final_exponentiation() == Gt::identity()
}
