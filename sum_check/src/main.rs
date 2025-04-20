use bls12_381::Scalar;
use ff::Field;
use std::time::Instant;

/// 0 または 1 のみを許す Scalar ラッパー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryScalar(Scalar);

impl BinaryScalar {
    /// 0 または 1 の Scalar からのみ生成可能
    pub fn new(value: Scalar) -> Self {
        if value == Scalar::ZERO || value == Scalar::ONE {
            BinaryScalar(value)
        } else {
            panic!("Invalid value: {}", value);
        }
    }

    /// 値を取り出す
    pub fn value(&self) -> Scalar {
        self.0
    }
}

// g = b_0 ** 0 + b_1 ** 1 + b_2 ** 2 + ...
fn g(b: &[BinaryScalar]) -> Scalar {
    let mut acc = Scalar::ZERO;

    for (i, bit) in b.iter().enumerate() {
        let pow = bit.value().pow(&[i as u64, 0, 0, 0]);
        acc += pow;
    }

    acc
}

fn h<F>(g: F, v: usize) -> Scalar
where
    F: Fn(&[BinaryScalar]) -> Scalar,
{
    let mut sum = Scalar::ZERO;
    let mut bits = vec![BinaryScalar::new(Scalar::ZERO); v];

    // 2^v 回のループを実行
    for i in 0..(1 << v) {
        // 現在の組み合わせを生成
        // iをbit表記した時に、1が立っているビットの位置に1を入れる
        for j in 0..v {
            if (i >> j) & 1 == 1 {
                bits[j] = BinaryScalar::new(Scalar::ONE);
            } else {
                bits[j] = BinaryScalar::new(Scalar::ZERO);
            }
        }

        // 現在の組み合わせで g を呼び出し、結果を合計に加える
        sum += g(&bits);
    }

    sum
}

fn main() {
    let start = Instant::now();
    println!("{}", h(g, 10));
    let duration = start.elapsed();
    println!("Time taken: {:?}", duration);
}
