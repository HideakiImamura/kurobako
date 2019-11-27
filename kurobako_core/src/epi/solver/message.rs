use crate::epi::ParamsForTransfer;
use crate::problem::ProblemSpec;
use crate::solver::SolverSpec;
use crate::trial::{EvaluatedTrial, NextTrial};
use crate::ErrorKind;
use serde::{Deserialize, Serialize};

/// Messages that are used to communicate with external solvers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolverMessage {
    SolverSpecCast {
        spec: SolverSpec,
    },
    CreateSolverCast {
        solver_id: u64,
        random_seed: u64,
        problem: ProblemSpec,
    },
    DropSolverCast {
        solver_id: u64,
    },
    AskCall {
        solver_id: u64,
        next_trial_id: u64,
    },
    AskReply {
        trial: NextTrial<ParamsForTransfer>,
        next_trial_id: u64,
    },
    TellCall {
        solver_id: u64,
        trial: EvaluatedTrial,
    },
    TellReply,
    ErrorReply {
        kind: ErrorKind,
        #[serde(default)]
        message: Option<String>,
    },
}
