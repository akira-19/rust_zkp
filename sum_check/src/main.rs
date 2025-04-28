use bls12_381::Scalar;
use ff::Field;
use rand::Rng;
use std::sync::Arc;

type GpFunc = Arc<dyn Fn(&[Scalar]) -> Scalar>;
type GiFunc = Arc<dyn Fn(Scalar) -> Scalar>;

macro_rules! duration {
    ($label:expr, $body:block) => {{
        let _start = std::time::Instant::now();
        let ret = { $body };
        println!("{} took {:?}", $label, _start.elapsed());
        ret
    }};
}

// g1(X1) = sigma g(X1, x2, x3...)
// g2(X2) = sigma g(r1, X2, x3...)
// rに何が来るか分からないが、hを計算するときに、xnから計算すれば、
// それぞれのg_iを事前に作っておける
// 例えば、g(X1, X2, X3, X4, X5)を計算する時に出てくる、
// gp(X) = g(X1, X2, X3, X4, 0) + g(X1, X2, X3, X4, 1)とする。（X = [X1, X2, X3, X4]）
// これを保存しておけば、X1からX3までのrandom値を取得したときに簡単に、
// g4(X4) = gp_5([r1, r2, r3, X4])と計算できる。

pub struct SumCheck {
    gp_is: Vec<GpFunc>,
    pub h: Scalar,
}

impl SumCheck {
    pub fn new() -> Self {
        Self {
            gp_is: Vec::new(),
            h: Scalar::ZERO,
        }
    }

    pub fn g(b: &[Scalar]) -> Scalar {
        // b.iter()
        //     .enumerate()
        //     .map(|(i, &val)| Scalar::from(i as u64) * val.pow(&[i as u64, 0, 0, 0]))
        //     .sum()
        b.iter().enumerate().fold(Scalar::ZERO, |acc, (i, &val)| {
            acc + Scalar::from(i as u64) * val.pow_vartime(&[i as u64, 0, 0, 0])
        })
    }

    /// gpᵢ(x₁,…,xᵢ) = gpᵢ₊₁(x₁,…,xᵢ,0) + gpᵢ₊₁(x₁,…,xᵢ,1)
    fn cal_gp_i(&self, g: GpFunc, i: usize) -> GpFunc {
        Arc::new(move |x: &[Scalar]| -> Scalar {
            assert_eq!(x.len(), i, "Expected {} args, got {}", i, x.len());

            // 1 回だけ確保し、最後の要素だけ書き換える
            let mut buf = Vec::with_capacity(i + 1);
            buf.extend_from_slice(x);
            buf.push(Scalar::ZERO);
            let mut sum = g(&buf);

            *buf.last_mut().unwrap() = Scalar::ONE;
            sum += g(&buf);

            sum
        })
    }

    pub fn gi_xi(&self, gp_i: Arc<dyn Fn(&[Scalar]) -> Scalar>, rs: &[Scalar]) -> GiFunc {
        let fixed_rs: Vec<Scalar> = rs.to_vec();

        Arc::new(move |x: Scalar| -> Scalar {
            let mut extended = Vec::with_capacity(fixed_rs.len() + 1);
            extended.extend_from_slice(&fixed_rs);
            extended.push(x);

            gp_i(&extended)
        })
    }

    /// h = gp₀() を返す
    pub fn cal_h<F>(&mut self, g: F, v: usize)
    where
        F: Fn(&[Scalar]) -> Scalar + 'static,
    {
        assert!(v > 0, "v must be ≥ 1");

        let mut gp_i: GpFunc = Arc::new(g);
        for i in (0..v).rev() {
            gp_i = self.cal_gp_i(gp_i, i);
            self.gp_is.insert(0, Arc::clone(&gp_i)); // gp₁ が index 0
        }
        self.h = gp_i(&[]);
    }
}

fn main() {
    let mut sumcheck = SumCheck::new();
    let p = 8;

    duration!("compute h", { sumcheck.cal_h(|x| SumCheck::g(x), p) });

    duration!("sumcheck protocol", {
        let mut rng = rand::rng();
        let mut rs = Vec::new();
        let mut gixi = sumcheck.gi_xi(sumcheck.gp_is[1].clone(), &[]);

        // ---------- ラウンド 1 ----------
        assert_eq!(sumcheck.h, gixi(Scalar::ZERO) + gixi(Scalar::ONE));

        // ---------- ラウンド 2 以降 ----------
        for i in 2..p {
            let r_i = Scalar::from(rng.random_range(1..10_000));
            rs.push(r_i);

            // gᵢ(x) を生成
            let gixi_next = sumcheck.gi_xi(sumcheck.gp_is[i].clone(), &rs);

            // 検証条件：gᵢ₋₁(rᵢ) = gᵢ(0) + gᵢ(1)
            assert_eq!(gixi(r_i), gixi_next(Scalar::ZERO) + gixi_next(Scalar::ONE));

            // 次ラウンドへ
            gixi = gixi_next;
        }
    });

    println!("{}", sumcheck.h);
}
