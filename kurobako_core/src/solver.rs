use crate::parameter::ParamValue;
use crate::problem::ProblemSpec;
use crate::time::Elapsed;
use crate::Result;
use rand::Rng;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGen, Obs};

pub trait SolverRecipe {
    type Solver: Solver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver>;
}

pub type UnobservedObs = Obs<Budgeted<Vec<ParamValue>>>;
pub type ObservedObs = Obs<Budgeted<Vec<ParamValue>>, Vec<FiniteF64>>;

pub trait Solver {
    fn specification(&self) -> SolverSpec;

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<Asked>;

    fn tell(&mut self, obs: ObservedObs) -> Result<Elapsed>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asked {
    pub obs: UnobservedObs,
    pub elapsed: Elapsed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SolverSpec {
    pub name: String,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub capabilities: SolverCapabilities,
}

pub type SolverCapabilities = BTreeSet<SolverCapability>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolverCapability {
    Categorical,
    Conditional,
    Discrete,
    MultiObjective,
}
