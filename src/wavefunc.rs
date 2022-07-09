use std::f32::consts::{E, PI};
use bacon_sci::polynomial; // Macro
use bacon_sci::polynomial::Polynomial;
use bacon_sci::special::{laguerre, legendre};
use nalgebra::{SVector, Complex, ComplexField};

type Cf32 = Complex<f32>;

fn fact_stirling(n: f32) -> f32 {
    if n <= 1.0 { 1.0 } else {
        (2.0*PI*n).sqrt()*(n/E).powf(n)
    }
}

fn derive(n: u32, poly: Polynomial<f32>) -> Polynomial<f32> {
    let mut poly_d = poly.clone();
    for _ in 1..=n {
        poly_d = poly_d.derivative();
    }
    poly_d
}

pub struct Psi {
    coeffs: [Cf32; 6],
    leg_poly: Polynomial<f32>,
    lag_poly: Polynomial<f32>,
}

impl Psi {
    pub fn new(n: u32, l: u32, m: u32) -> Self {
        let p = 2*l+1;
        let q = n-l-1;
        let r = 2.0/n as f32;
        let s = fact_stirling((n+l) as f32);
        let t = fact_stirling(q as f32);
        let u = (r.powf(3.0)/(2.0*(s/t)*n as f32)).sqrt();
        let v = fact_stirling((l-m) as f32);
        let w = fact_stirling((l+m) as f32);
        let o = (v/(w*4.0*PI)*p as f32).sqrt();
        let k = 0.5*m as f32;
        let tol = 1e-8;

        let coeffs = [r,u,k,o,l as f32,m as f32].map(Complex::from);

        let scalar_leg = polynomial![(-1.0).powf(m as f32)];
        let scalar_lag = polynomial![(-1.0).powf(p as f32)];

        Self {
            coeffs,
            leg_poly: scalar_leg*derive(m, legendre(l, tol).unwrap()),
            lag_poly: scalar_lag*derive(p, laguerre(p+q, tol).unwrap()),
        }
    }

    pub fn eval<const D: usize> (
        &self,
        x: &SVector<f32, D>,
        y: &SVector<f32, D>,
        z: &SVector<f32, D>,
    ) -> SVector<Cf32, D> {
        let r_2 = x.component_mul(x)+y.component_mul(y);
        let r_3 = (r_2+z.component_mul(z)).map(|i| i.sqrt());
        let rho = r_3.map(|i| i.abs()*self.coeffs[0].re);
        let r_nl = rho.map(|i| self.lag_poly.evaluate(i)*self.coeffs[1].re)
                    .component_mul(&rho.map(|i| i.powf(self.coeffs[4].re)))
                    .component_mul(&rho.map(|i| (-i/2.0).exp()));
        let cos_theta = z.component_div(&r_3)
                    .map(|i| if i.abs() >= 1.0 {i.signum()*1.0} else {i})
                    .map(|i| self.leg_poly.evaluate(i)*(1.0-i*i).powf(self.coeffs[2].re));
        unsafe {
            let ei_mtheta = SVector::<Cf32, D>::from_fn(|i, _| Cf32::from(y.vget_unchecked(i))*Cf32::i())
                        .map_with_location(|i, _, v|
                            (v+Cf32::from(x.vget_unchecked(i)))/Cf32::from(r_2.vget_unchecked(i)).sqrt())
                        .map(|i| i.powf(self.coeffs[5].re));
            let y_lm = ei_mtheta.map_with_location(|i, _, v|
                        v*Cf32::from(cos_theta.vget_unchecked(i))*self.coeffs[3]);

            y_lm.map_with_location(|i, _, v| v*Cf32::from(r_nl.vget_unchecked(i)))
        }
    }
}
