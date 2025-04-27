use bls12_381::Scalar;
use ff::Field;
use rand::Rng;
use std::sync::Arc;
use std::time::Instant;
use std::vec;

// g1(X1) = sigma g(X1, x2, x3...)
// g2(X2) = sigma g(r1, X2, x3...)
// rに何が来るか分からないが、hを計算するときに、xnから計算すれば、
// それぞれのg_iを事前に作っておける
// 例えば、g(X1, X2, X3, X4, X5)を計算する時に出てくる、
// gp(X) = g(X1, X2, X3, X4, 0) + g(X1, X2, X3, X4, 1)とする。（X = [X1, X2, X3, X4]）
// これを保存しておけば、X1からX3までのrandom値を取得したときに簡単に、
// g4(X4) = gp_5([r1, r2, r3, X4])と計算できる。

pub struct SumCheck {
    gp_is: Vec<Arc<dyn Fn(&[Scalar]) -> Scalar>>,
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
        b.iter()
            .enumerate()
            .map(|(i, &val)| Scalar::from(i as u64) * val.pow(&[i as u64, 0, 0, 0]))
            .sum()
    }

    /// gpᵢ(x₁,…,xᵢ) = gpᵢ₊₁(x₁,…,xᵢ,0) + gpᵢ₊₁(x₁,…,xᵢ,1)
    fn cal_gp_i(
        &self,
        g: Arc<dyn Fn(&[Scalar]) -> Scalar>,
        i: usize,
    ) -> Arc<dyn Fn(&[Scalar]) -> Scalar> {
        Arc::new(move |x: &[Scalar]| -> Scalar {
            assert_eq!(x.len(), i, "Expected {} args, got {}", i, x.len());

            let mut tmp = x.to_vec();
            tmp.push(Scalar::ZERO);
            let mut sum = g(&tmp);

            tmp[i] = Scalar::ONE;
            sum += g(&tmp);

            sum
        })
    }

    pub fn gi_xi(
        &self,
        gp_i: Arc<dyn Fn(&[Scalar]) -> Scalar>,
        rs: &[Scalar],
    ) -> Arc<dyn Fn(Scalar) -> Scalar> {
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

        let mut i = v - 1;
        let mut gp_i: Arc<dyn Fn(&[Scalar]) -> Scalar> = Arc::new(g);

        while i > 0 {
            gp_i = self.cal_gp_i(gp_i, i);
            self.gp_is.insert(0, Arc::clone(&gp_i));
            i -= 1;
        }

        gp_i = self.cal_gp_i(gp_i, i);
        self.gp_is.insert(0, Arc::clone(&gp_i));

        let h = gp_i(&[]);
        self.h = h;
    }
}

fn main() {
    let mut sumcheck = SumCheck::new();
    let p = 13;

    let start = Instant::now();
    // pがhを計算
    sumcheck.cal_h(|x| SumCheck::g(x), p);

    let duration = start.elapsed();
    println!("Time taken by calc h: {:?}", duration);

    let start = Instant::now();

    let gp_1 = sumcheck.gp_is[1].clone();
    let g1x1 = sumcheck.gi_xi(gp_1, &[]);

    assert_eq!(sumcheck.h, g1x1(Scalar::ZERO) + g1x1(Scalar::ONE));

    let mut rs = vec![];
    let mut last_gixi = g1x1;

    for i in 2..p {
        let mut rng = rand::rng();
        let r_i = Scalar::from(rng.random_range(1..10000));
        rs.push(r_i);
        let gp_i = sumcheck.gp_is[i].clone();
        let gixi = sumcheck.gi_xi(gp_i, &rs);

        assert_eq!(last_gixi(r_i), gixi(Scalar::ZERO) + gixi(Scalar::ONE));

        last_gixi = gixi;
    }

    let duration = start.elapsed();
    println!("Time taken by sumcheck: {:?}", duration);

    // pub fn gi_xi(
    //     &self,
    //     gp_i: Arc<dyn Fn(&[Scalar]) -> Scalar>,
    //     rs: &[Scalar],
    // )

    println!("{}", sumcheck.h);
}
