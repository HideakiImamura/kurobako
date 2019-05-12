use crate::problem::KurobakoProblemRecipe;
use kurobako_core::parameter::{uniform, ParamDomain, ParamValue};
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, Evaluate, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::solver::{Solver, SolverRecipe, SolverSpec};
use kurobako_core::{ErrorKind, Result};
use kurobako_solvers::optuna::{OptunaSolver, OptunaSolverRecipe};
use rand;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::num::NonZeroU64;
use std::rc::Rc;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::{ObsId, SerialIdGenerator};

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct OptunaProblemRecipe {
    #[structopt(long, default_value = "100")]
    pub budget: u64,

    #[structopt(flatten)]
    #[serde(flatten)]
    pub optuna: OptunaSolverRecipe,

    #[structopt(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tpe_gamma_factor_min: Option<f64>,

    #[structopt(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tpe_gamma_factor_max: Option<f64>,

    #[structopt(subcommand)]
    pub problem: KurobakoProblemRecipe,
}
impl ProblemRecipe for OptunaProblemRecipe {
    type Problem = OptunaProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let problem = track!(self.problem.create_problem())?;
        let solver = track!(self.optuna.create_solver(problem.specification()))?.specification();
        let budget = track_assert_some!(NonZeroU64::new(self.budget), ErrorKind::InvalidInput);

        let mut params_domain = Vec::new();
        if self.tpe_gamma_factor_min.is_some() && self.tpe_gamma_factor_max.is_some() {
            params_domain.push(track!(uniform(
                "tpe_gamma_factor",
                self.tpe_gamma_factor_min.unwrap(),
                self.tpe_gamma_factor_max.unwrap()
            ))?);
        }

        Ok(OptunaProblem {
            problem: Rc::new(RefCell::new(problem)),
            solver,
            optuna: self.optuna.clone(),
            budget,
            params_domain,
        })
    }
}

#[derive(Debug)]
pub struct OptunaProblem {
    problem: Rc<RefCell<BoxProblem>>,
    solver: SolverSpec,
    optuna: OptunaSolverRecipe,
    budget: NonZeroU64,
    params_domain: Vec<ParamDomain>,
}
impl Problem for OptunaProblem {
    type Evaluator = OptunaEvaluator;

    fn specification(&self) -> ProblemSpec {
        let problem = self.problem.borrow().specification();
        ProblemSpec {
            name: self.solver.name.clone(),
            version: self.solver.version.clone(),
            params_domain: self.params_domain.clone(),
            values_domain: problem.values_domain,
            evaluation_expense: unsafe {
                NonZeroU64::new_unchecked(self.budget.get() * problem.evaluation_expense.get())
            },
            capabilities: Default::default(), // TODO
        }
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        Ok(OptunaEvaluator {
            optuna: self.optuna.clone(),
            solver: None,
            problem: self.problem.clone(),
            idg: SerialIdGenerator::new(),
            curr_id: None,
            evaluator: None,
            best_values: None,
            params_domain: self.params_domain.clone(),
        })
    }
}

#[derive(Debug)]
pub struct OptunaEvaluator {
    optuna: OptunaSolverRecipe,
    solver: Option<OptunaSolver>,
    problem: Rc<RefCell<BoxProblem>>,
    idg: SerialIdGenerator,
    curr_id: Option<ObsId>,
    evaluator: Option<BoxEvaluator>,
    best_values: Option<Vec<FiniteF64>>,
    params_domain: Vec<ParamDomain>,
}
impl OptunaEvaluator {
    fn evaluate_once(&mut self) -> Result<(Values, u64)> {
        let mut rng = rand::thread_rng(); // TODO

        let mut asked_obs = track!(self.solver.as_mut().unwrap().ask(&mut rng, &mut self.idg))?;
        if Some(asked_obs.id) != self.curr_id {
            // TODO: handle cuncurrent
            self.curr_id = Some(asked_obs.id);
            self.evaluator = Some(track!(self
                .problem
                .borrow_mut()
                .create_evaluator(asked_obs.id))?);
        }

        let mut budget = asked_obs.param.budget();
        let old_consumption = budget.consumption;
        let evaluated_values = track!(self
            .evaluator
            .as_mut()
            .unwrap()
            .evaluate(asked_obs.param.get(), &mut budget))?;
        let delta_consumption = budget.consumption - old_consumption;

        *asked_obs.param.budget_mut() = budget;
        let obs = asked_obs.map_value(|()| evaluated_values.clone());
        track!(self.solver.as_mut().unwrap().tell(obs))?;

        Ok((evaluated_values, delta_consumption))
    }
}
impl Evaluate for OptunaEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        if self.solver.is_none() {
            for (name, p) in self
                .params_domain
                .iter()
                .map(|p| p.name())
                .zip(params.iter())
            {
                if name == "tpe_gamma_factor" {
                    if let ParamValue::Continuous(v) = p {
                        self.optuna.tpe_gamma_factor = v.get();
                    }
                }
            }
            self.solver = Some(track!(self
                .optuna
                .create_solver(self.problem.borrow().specification()))?);
        }

        while !budget.is_consumed() {
            let (evaluated_values, consumption) = track!(self.evaluate_once())?;
            budget.consumption += consumption;

            // TODO: support multi-objective (in such case, the order cannot be uniquely determined)
            if self.best_values.is_none() || &evaluated_values < self.best_values.as_ref().unwrap()
            {
                self.best_values = Some(evaluated_values);
            }
        }
        Ok(track_assert_some!(
            self.best_values.clone().take(),
            ErrorKind::Bug
        ))
    }
}
