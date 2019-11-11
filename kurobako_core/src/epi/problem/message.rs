use crate::problem::ProblemSpec;
use crate::trial::{Params, Values};
use crate::ErrorKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemMessage {
    ProblemSpecCast {
        spec: ProblemSpec,
    },
    CreateProblemCast {
        problem_id: u64,
        random_seed: u64,
    },
    DropProblemCast {
        problem_id: u64,
    },
    CreateEvaluatorCall {
        problem_id: u64,
        evaluator_id: u64,
        params: Params,
    },
    CreateEvaluatorOkReply,
    DropEvaluatorCast {
        problem_id: u64,
        evaluator_id: u64,
    },
    EvaluateCall {
        problem_id: u64,
        evaluator_id: u64,
        next_step: u64,
    },
    EvaluateOkReply {
        current_step: u64,
        values: Values,
    },
    ErrorReply {
        kind: ErrorKind,
        #[serde(default)]
        message: Option<String>,
    },
}
