
use ark_ff::{
    Field, MontFp, Zero,
    field_hashers::{DefaultFieldHasher, HashToField},
};

use ark_ec::{AffineRepr, CurveGroup};

use ark_bls12_381::{Config, Fq, G1Affine};

pub fn bls12_381_hash_to_g1(msg: &[u8], dst: &str) -> G1Affine {
    use Fq;
    let htf = <DefaultFieldHasher<sha2::Sha256, 128> as HashToField<Fq>>::new(dst.as_bytes());
    let u = htf.hash_to_field(msg, 2);
    let (x, y) = map_to_curve_simple_swu(u[0]);
    let (x, y) = iso_map_swu(x, y);
    let p0 = G1Affine::new_unchecked(x, y);
    let (x, y) = map_to_curve_simple_swu(u[1]);
    let (x, y) = iso_map_swu(x, y);
    let p1 = G1Affine::new_unchecked(x, y);
    (p0 + p1).into_affine().clear_cofactor()
}

static Z: Fq = MontFp!("11");
static A: Fq = MontFp!("12190336318893619529228877361869031420615612348429846051986726275283378313155663745811710833465465981901188123677");
static B: Fq = MontFp!("2906670324641927570491258158026293881577086121416628140204402091718288198173574630967936031029026176254968826637280");

fn map_to_curve_simple_swu(u: Fq) -> (Fq, Fq) {
    // Simplified SWU map for BLS12-381 G1
    // See https://www.rfc-editor.org/rfc/rfc9380.html#straightline-sswu
    // 1.  tv1 = u^2
    let mut tv1 = u.square();
    // 2.  tv1 = Z * tv1
    tv1 = Z * tv1;
    // 3.  tv2 = tv1^2
    let mut tv2 = tv1.square();
    // 4.  tv2 = tv2 + tv1
    tv2 += tv1;
    // 5.  tv3 = tv2 + 1
    let mut tv3 = tv2 + Fq::ONE;
    // 6.  tv3 = B * tv3
    tv3 = B * tv3;
    // 7.  tv4 = CMOV(Z, -tv2, tv2 != 0)
    let mut tv4 = if !tv2.is_zero() { -tv2 } else { Z };
    // 8.  tv4 = A * tv4
    tv4 *= A;
    // 9.  tv2 = tv3^2
    tv2 = tv3.square();
    // 10. tv6 = tv4^2
    let mut tv6 = tv4.square();
    // 11. tv5 = A * tv6
    let tv5 = A * tv6;
    // 12. tv2 = tv2 + tv5
    tv2 += tv5;
    // 13. tv2 = tv2 * tv3
    tv2 *= tv3;
    // 14. tv6 = tv6 * tv4
    tv6 *= tv4;
    // 15. tv5 = B * tv6
    let tv5 = B * tv6;
    // 16. tv2 = tv2 + tv5
    tv2 += tv5;
    // 17.   x = tv1 * tv3
    let mut x = tv1 * tv3;
    // 18. (is_gx1_square, y1) = sqrt_ratio(tv2, tv6)
    let (is_gx1_square, y1) = sqrt_ratio(tv2, tv6);
    // 19.   y = tv1 * u
    let mut y = tv1 * u;
    // 20.   y = y * y1
    y *= y1;
    // 21.   x = CMOV(x, tv3, is_gx1_square)
    x = if !is_gx1_square { x } else { tv3 };
    // 22.   y = CMOV(y, y1, is_gx1_square)
    y = if !is_gx1_square { y } else { y1 };
    // 23.  e1 = sgn0(u) == sgn0(y)
    let e1 = sgn0(u) == sgn0(y);
    // 24.   y = CMOV(-y, y, e1)
    if !e1 { y.neg_in_place(); }
    // 25.   x = x / tv4
    x /= tv4;
    // 26. return (x, y)
    (x, y)
}

/// https://www.rfc-editor.org/rfc/rfc9380.html#name-the-sgn0-function
fn sgn0(x: Fq) -> bool {
    use ark_ff::BigInteger;
    use ark_ff::PrimeField;
    x.into_bigint().is_odd()
}

fn sqrt_ratio(u: Fq, v: Fq) -> (bool, Fq) {
    // simple naive implementation, not based on the optimized one from the RFC
    if let Some(root) = (u / v).sqrt() {
        (true, root)
    } else {
        let root = (u * Z / v).sqrt().expect("assuming Z is a non-residue");
        (false, root)
    }
}

/// copypasted from arkworks
fn iso_map_swu(x: Fq, y: Fq) -> (Fq, Fq) {
    use ark_ec::hashing::curve_maps::wb::WBConfig;
    use ark_ec::models::bls12::Bls12Config;
    use ark_ff::batch_inversion;
    use ark_poly::DenseUVPolynomial;
    use ark_poly::Polynomial;
    use ark_poly::univariate::DensePolynomial;

    let iso = <Config as Bls12Config>::G1Config::ISOGENY_MAP;
    // NOTE: identity point not handled
    let x_num = DensePolynomial::from_coefficients_slice(iso.x_map_numerator);
    let x_den = DensePolynomial::from_coefficients_slice(iso.x_map_denominator);

    let y_num = DensePolynomial::from_coefficients_slice(iso.y_map_numerator);
    let y_den = DensePolynomial::from_coefficients_slice(iso.y_map_denominator);

    let mut v = [x_den.evaluate(&x), y_den.evaluate(&x)];
    batch_inversion(&mut v);
    let img_x = x_num.evaluate(&x) * v[0];
    let img_y = (y_num.evaluate(&x) * y) * v[1];
    (img_x, img_y)
}
