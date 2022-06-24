use std::f64::consts::{E, PI};
use bacon_sci::polynomial; // Macro
use bacon_sci::polynomial::Polynomial;
use bacon_sci::special::{laguerre, legendre};
use nalgebra::{SVector, Complex, ComplexField};

type Cf64 = Complex<f64>;

fn fact_stirling(n: f64) -> f64 {
    if n <= 1.0 { 1.0 } else {
        (2.0*PI*n).sqrt()*(n/E).powf(n)
    }
}

fn derive(n: u32, poly: Polynomial<f64>) -> Polynomial<f64> {
    let mut poly_d = poly.clone();
    for _ in 1..=n {
        poly_d = poly_d.derivative();
    }
    poly_d
}

pub struct Psi {
    coeffs: [Cf64; 6],
    leg_poly: Polynomial<f64>,
    lag_poly: Polynomial<f64>,
}

impl Psi {
    pub fn new(n: u32, l: u32, m: u32) -> Self {
        let p = 2*l+1;
        let q = n-l-1;
        let r = 2.0/n as f64;
        let s = fact_stirling((n+l) as f64);
        let t = fact_stirling(q as f64);
        let u = (r.powf(3.0)/(2.0*(s/t)*n as f64)).sqrt();
        let v = fact_stirling((l-m) as f64);
        let w = fact_stirling((l+m) as f64);
        let o = (v/(w*4.0*PI)*p as f64).sqrt();
        let k = 0.5*m as f64;
        let tol = 1e-8;

        let coeffs = [r,u,k,o,l as f64,m as f64].map(Complex::from);

        let scalar_leg = polynomial![(-1.0).powf(m as f64)];
        let scalar_lag = polynomial![(-1.0).powf(p as f64)];

        Self {
            coeffs,
            leg_poly: scalar_leg*derive(m, legendre(l, tol).unwrap()),
            lag_poly: scalar_lag*derive(p, laguerre(p+q, tol).unwrap()),
        }
    }

    pub fn eval<const D: usize> (
        &self,
        x: &SVector<f64, D>,
        y: &SVector<f64, D>,
        z: &SVector<f64, D>,
    ) -> SVector<Cf64, D> {
        let r_2 = x.component_mul(x)+y.component_mul(y);
        let r_3 = (r_2+z.component_mul(z)).map(|i| i.sqrt());
        let rho = r_3.map(|i| i.abs()*self.coeffs[0].re);
        let r_nl = rho.map(|i| self.lag_poly.evaluate(i)*self.coeffs[1].re)
                    .component_mul(&rho.map(|i| i.powf(self.coeffs[4].re)))
                    .component_mul(&rho.map(|i| (-i/2.0).exp()));
        let cos_theta = z.component_div(&r_3)
                    .map(|i| if i.abs() >= 1.0 {i.signum()*i/i} else {i})
                    .map(|i| self.leg_poly.evaluate(i)*(1.0-i*i).powf(self.coeffs[2].re));
        let ei_mtheta = SVector::<Cf64, D>::from_fn(|i, _| Cf64::from(y.index(i))*Cf64::i())
                    .map_with_location(|i, _, v|
                        (v+Cf64::from(x.index(i)))/Cf64::from(r_2.index(i)).sqrt())
                    .map(|i| i.powf(self.coeffs[5].re));
        let y_lm = ei_mtheta.map_with_location(|i, _, v|
            v*Cf64::from(cos_theta.index(i))*self.coeffs[3]*Cf64::from(r_nl.index(i)));

        y_lm
    }
}
