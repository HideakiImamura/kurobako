use crate::distribution::Distribution;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::ops::Range;
use structopt::StructOpt;
use yamakan::ParamSpace;

pub use self::adjiman::AdjimanProblem;
pub use self::alpine::{Alpine01Problem, Alpine02Problem};

mod adjiman;
mod alpine;

pub trait Problem: StructOpt + Serialize + for<'a> Deserialize<'a> {
    fn name(&self) -> &str;
    fn problem_space(&self) -> ProblemSpace;
    fn evaluate(&self, params: &[f64]) -> f64;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ProblemSpec {
    Ackley(AckleyProblem),
    Adjiman(AdjimanProblem),
    Alpine01(Alpine01Problem),
    Alpine02(Alpine02Problem),
}
impl Problem for ProblemSpec {
    fn name(&self) -> &str {
        match self {
            ProblemSpec::Ackley(x) => x.name(),
            ProblemSpec::Adjiman(x) => x.name(),
            ProblemSpec::Alpine01(x) => x.name(),
            ProblemSpec::Alpine02(x) => x.name(),
        }
    }

    fn problem_space(&self) -> ProblemSpace {
        match self {
            ProblemSpec::Ackley(x) => x.problem_space(),
            ProblemSpec::Adjiman(x) => x.problem_space(),
            ProblemSpec::Alpine01(x) => x.problem_space(),
            ProblemSpec::Alpine02(x) => x.problem_space(),
        }
    }

    fn evaluate(&self, params: &[f64]) -> f64 {
        match self {
            ProblemSpec::Ackley(x) => x.evaluate(params),
            ProblemSpec::Adjiman(x) => x.evaluate(params),
            ProblemSpec::Alpine01(x) => x.evaluate(params),
            ProblemSpec::Alpine02(x) => x.evaluate(params),
        }
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct AckleyProblem {
    #[structopt(long, default_value = "2")]
    pub dim: usize,
}
impl Problem for AckleyProblem {
    fn name(&self) -> &str {
        "ackley"
    }

    fn problem_space(&self) -> ProblemSpace {
        ProblemSpace(
            (0..self.dim)
                .map(|_| Distribution::Uniform {
                    low: -10.0,
                    high: 30.0,
                })
                .collect(),
        )
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = self.dim as f64;
        let a = 20.0;
        let b = 0.2;
        let c = 2.0 * PI;
        let d = -a * (-b * (1.0 / dim * xs.iter().map(|&x| x.powi(2)).sum::<f64>()).sqrt()).exp();
        let e = (1.0 / dim * xs.iter().map(|&x| (x * c).cos()).sum::<f64>()).exp();
        let f = a + 1f64.exp();
        d - e + f
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemSpace(Vec<Distribution>);
impl ProblemSpace {
    pub fn distributions(&self) -> &[Distribution] {
        &self.0
    }
}
impl ParamSpace for ProblemSpace {
    type External = Vec<f64>;
    type Internal = Vec<f64>;

    fn internal_range(&self) -> Range<Self::Internal> {
        Range {
            start: self.0.iter().map(|d| d.low()).collect(),
            end: self.0.iter().map(|d| d.high()).collect(),
        }
    }

    fn internalize(&self, param: &Self::External) -> Self::Internal {
        param.clone()
    }

    fn externalize(&self, param: &Self::Internal) -> Self::External {
        param.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ackley_works() {
        let p = AckleyProblem { dim: 2 };
        let v = p.evaluate(&[-0.991579880560538, 0.7860986559165095]);
        assert_eq!(v, 4.151720074504926);
    }
}
