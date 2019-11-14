//! Problem interface for black-box optimization.
use crate::domain::{Distribution, Domain, Range, VariableBuilder};
use crate::repository::Repository;
use crate::rng::ArcRng;
use crate::solver::Capabilities;
use crate::trial::{Params, Values};
use crate::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU64;
use structopt::StructOpt;

/// `ProblemSpec` builder.
#[derive(Debug)]
pub struct ProblemSpecBuilder {
    name: String,
    attrs: BTreeMap<String, String>,
    params: Vec<VariableBuilder>,
    values: Vec<VariableBuilder>,
    evaluation_steps: u64,
}
impl ProblemSpecBuilder {
    /// Makes a new `ProblemSpecBuilder` instance.
    pub fn new(problem_name: &str) -> Self {
        Self {
            name: problem_name.to_owned(),
            attrs: BTreeMap::new(),
            params: Vec::new(),
            values: Vec::new(),
            evaluation_steps: 1,
        }
    }

    /// Sets an attribute of this problem.
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_owned(), value.to_owned());
        self
    }

    /// Adds a variable to the parameter domain of this problem.
    pub fn param(mut self, var: VariableBuilder) -> Self {
        self.params.push(var);
        self
    }

    /// Adds a variable to the value domain of this problem.
    pub fn value(mut self, var: VariableBuilder) -> Self {
        self.values.push(var);
        self
    }

    /// Sets the evaluation steps of this problem.
    pub fn evaluation_steps(mut self, steps: u64) -> Self {
        self.evaluation_steps = steps;
        self
    }

    /// Builds a `ProblemSpec` with the given settings.
    pub fn finish(self) -> Result<ProblemSpec> {
        let params_domain = track!(Domain::new(self.params))?;
        let values_domain = track!(Domain::new(self.values))?;
        let evaluation_steps = track_assert_some!(
            NonZeroU64::new(self.evaluation_steps),
            ErrorKind::InvalidInput
        );

        Ok(ProblemSpec {
            name: self.name,
            attrs: self.attrs,
            params_domain,
            values_domain,
            evaluation_steps,
        })
    }
}

/// Problem specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemSpec {
    /// Problem name.
    pub name: String,

    /// Problem attributes.
    #[serde(default)]
    pub attrs: BTreeMap<String, String>,

    /// Domain of the parameters.
    pub params_domain: Domain,

    /// Domain of the objective values.
    pub values_domain: Domain,

    /// Number of steps to complete evaluating a parameter set.
    pub evaluation_steps: NonZeroU64,
}
impl ProblemSpec {
    /// Returns the capabilities required to solver to handle this problem.
    pub fn requirements(&self) -> Capabilities {
        let mut c = Capabilities::empty();

        if self.values_domain.variables().len() > 1 {
            c = c.multi_objective();
        }

        for v in self.params_domain.variables() {
            if !v.conditions().is_empty() {
                c = c.conditional();
            }

            match (v.range(), v.distribution()) {
                (Range::Continuous { .. }, Distribution::Uniform) => {
                    c = c.uniform_continuous();
                }
                (Range::Continuous { .. }, Distribution::LogUniform) => {
                    c = c.log_uniform_continuous();
                }
                (Range::Discrete { .. }, Distribution::Uniform) => {
                    c = c.uniform_discrete();
                }
                (Range::Discrete { .. }, Distribution::LogUniform) => {
                    c = c.log_uniform_discrete();
                }
                (Range::Categorical { .. }, _) => {
                    c = c.categorical();
                }
            }
        }

        c
    }
}

pub trait ProblemRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Factory: ProblemFactory;

    fn create_factory(&self, repository: &mut Repository) -> Result<Self::Factory>;
}

pub trait ProblemFactory {
    type Problem: Problem;

    fn specification(&self) -> Result<ProblemSpec>;
    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem>;
}

enum ProblemFactoryCall {
    Specification,
    CreateProblem(ArcRng),
}

enum ProblemFactoryReturn {
    Specification(ProblemSpec),
    CreateProblem(BoxProblem),
}

pub struct BoxProblemFactory(Box<dyn Fn(ProblemFactoryCall) -> Result<ProblemFactoryReturn>>);
impl BoxProblemFactory {
    pub fn new<T>(problem: T) -> Self
    where
        T: ProblemFactory + 'static,
    {
        Self(Box::new(move |call| match call {
            ProblemFactoryCall::Specification => problem
                .specification()
                .map(ProblemFactoryReturn::Specification),
            ProblemFactoryCall::CreateProblem(rng) => problem
                .create_problem(rng)
                .map(BoxProblem::new)
                .map(ProblemFactoryReturn::CreateProblem),
        }))
    }
}
impl ProblemFactory for BoxProblemFactory {
    type Problem = BoxProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let v = track!((self.0)(ProblemFactoryCall::Specification))?;
        if let ProblemFactoryReturn::Specification(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let v = track!((self.0)(ProblemFactoryCall::CreateProblem(rng)))?;
        if let ProblemFactoryReturn::CreateProblem(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }
}
impl fmt::Debug for BoxProblemFactory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxProblemFactory {{ .. }}")
    }
}

pub trait Problem {
    type Evaluator: Evaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator>;
}

pub struct BoxProblem(Box<dyn Fn(Params) -> Result<BoxEvaluator>>);
impl BoxProblem {
    pub fn new<T>(problem: T) -> Self
    where
        T: Problem + 'static,
    {
        Self(Box::new(move |params| {
            problem.create_evaluator(params).map(BoxEvaluator::new)
        }))
    }
}
impl Problem for BoxProblem {
    type Evaluator = BoxEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        track!((self.0)(params))
    }
}
impl fmt::Debug for BoxProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxProblem {{ .. }}")
    }
}

pub trait Evaluator {
    fn evaluate(&mut self, max_step: u64) -> Result<(u64, Values)>;
}
impl<T: Evaluator + ?Sized> Evaluator for Box<T> {
    fn evaluate(&mut self, max_step: u64) -> Result<(u64, Values)> {
        (**self).evaluate(max_step)
    }
}

pub struct BoxEvaluator(Box<(dyn Evaluator + 'static)>);
impl BoxEvaluator {
    pub fn new<T>(evaluator: T) -> Self
    where
        T: Evaluator + 'static,
    {
        Self(Box::new(evaluator))
    }
}
impl Evaluator for BoxEvaluator {
    fn evaluate(&mut self, max_step: u64) -> Result<(u64, Values)> {
        self.0.evaluate(max_step)
    }
}
impl fmt::Debug for BoxEvaluator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxEvaluator {{ .. }}")
    }
}
