
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
    let tv1 = (Z.square() * u.square().square()) + (Z * u.square());
    let x1 = if tv1.is_zero() {
        B * (Z * A).inverse().unwrap_or(Fq::zero())
    } else {
        (-B / A) * (Fq::ONE + tv1.inverse().unwrap_or(Fq::zero()))
    };
    let gx1 = x1.square() * x1 + A * x1 + B;
    let x2 = Z * u.square() * x1;
    let gx2 = x2.square() * x2 + A * x2 + B;
    let (x, mut y) = if let Some(y1) = gx1.sqrt() {
        (x1, y1)
    } else {
        let y2 = gx2.sqrt().expect("assuming gx2 is square");
        (x2, y2)
    };
    if sgn0(u) != sgn0(y) {
        y.neg_in_place();
    }
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
