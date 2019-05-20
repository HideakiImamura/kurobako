use super::{JsonValue, TrialRecord};
use crate::runner::StudyRunnerOptions;
use crate::time::DateTime;
use chrono::Local;
use kurobako_core::problem::{ProblemRecipe, ProblemSpec};
use kurobako_core::solver::{SolverRecipe, SolverSpec};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAndSpec<T> {
    pub spec: T,
    pub recipe: JsonValue, // TODO: FullKurobakoProblemRecipe or KurobakoSolverRecipe
}
impl RecipeAndSpec<ProblemSpec> {
    pub fn id(&self) -> (&String, Option<&String>, &JsonValue) {
        (&self.spec.name, self.spec.version.as_ref(), &self.recipe)
    }
}
impl RecipeAndSpec<SolverSpec> {
    pub fn id(&self) -> (&String, Option<&String>, &JsonValue) {
        (&self.spec.name, self.spec.version.as_ref(), &self.recipe)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub solver: RecipeAndSpec<SolverSpec>,
    pub problem: RecipeAndSpec<ProblemSpec>,
    pub runner: StudyRunnerOptions,
    pub start_time: DateTime,
    pub end_time: DateTime,
    pub unevaluable_trials: usize,
    pub trials: Vec<TrialRecord>,
}
impl StudyRecord {
    pub fn new<O, P>(
        solver_recipe: &O,
        solver_spec: SolverSpec,
        problem_recipe: &P,
        problem_spec: ProblemSpec,
        runner: StudyRunnerOptions,
    ) -> Result<Self>
    where
        O: SolverRecipe,
        P: ProblemRecipe,
    {
        let solver = RecipeAndSpec {
            recipe: JsonValue::new(track!(
                serde_json::to_value(solver_recipe).map_err(Error::from)
            )?),
            spec: solver_spec,
        };
        let problem = RecipeAndSpec {
            recipe: JsonValue::new(track!(
                serde_json::to_value(problem_recipe).map_err(Error::from)
            )?),
            spec: problem_spec,
        };
        Ok(StudyRecord {
            solver,
            problem,
            runner,
            start_time: Local::now(),
            end_time: Local::now(), // dummy value
            unevaluable_trials: 0,
            trials: Vec::new(),
        })
    }

    pub fn finish(&mut self) {
        self.end_time = Local::now();
    }

    pub fn study_budget(&self) -> u64 {
        self.runner.budget * self.trial_budget()
    }

    pub fn trial_budget(&self) -> u64 {
        self.problem.spec.evaluation_expense.get()
    }

    pub fn scorer(&self) -> Scorer {
        Scorer::new(self)
    }
}

#[derive(Debug)]
pub struct Scorer {
    bests: Vec<(u64, f64)>,
}
impl Scorer {
    fn new(study: &StudyRecord) -> Self {
        let mut trials = HashMap::<ObsId, u64>::new();
        let mut consumption = 0;
        let mut bests: Vec<(u64, f64)> = Vec::new();
        for trial in &study.trials {
            *trials.entry(trial.obs_id).or_default() += trial.evaluate.expense;
            consumption += trial.evaluate.expense;

            if trials[&trial.obs_id] >= study.trial_budget() {
                let value = trial.evaluate.values[0].get();
                if bests.is_empty() || Some(value) <= bests.last().map(|t| t.1) {
                    let consumption = if bests.is_empty() { 0 } else { consumption }; // TODO: remove
                    bests.push((consumption, value));
                }
            }
        }

        Self { bests }
    }

    // TODO: return Option<f64>
    pub fn best_value(&self, budget: u64) -> f64 {
        self.bests
            .iter()
            .take_while(|t| t.0 <= budget)
            .map(|t| t.1)
            .last()
            .unwrap()
    }
}
