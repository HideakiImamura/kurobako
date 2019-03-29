use crate::time::{Stopwatch, Timestamp};
use crate::Result;
use yamakan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub ask: AskRecord,
    pub evals: Vec<EvalRecord>,
    pub complete: bool,
}
impl TrialRecord {
    pub fn value(&self) -> Option<f64> {
        self.evals.last().map(|x| x.value)
    }

    pub fn end_time(&self) -> Timestamp {
        self.evals
            .last()
            .map_or(Timestamp::new(0.0), |x| x.end_time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskRecord {
    pub params: Vec<f64>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
impl AskRecord {
    pub fn with<F>(watch: &Stopwatch, f: F) -> Result<Self>
    where
        F: FnOnce() -> yamakan::Result<Vec<f64>>,
    {
        let start_time = watch.elapsed();
        let params = f()?;
        let end_time = watch.elapsed();
        Ok(Self {
            params,
            start_time,
            end_time,
        })
    }

    pub fn latency(&self) -> f64 {
        self.end_time.as_seconds() - self.start_time.as_seconds()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRecord {
    pub value: f64,
    pub cost: usize,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
impl EvalRecord {
    pub fn with<F>(watch: &Stopwatch, f: F) -> Self
    where
        F: FnOnce() -> f64,
    {
        let start_time = watch.elapsed();
        let value = f();
        let end_time = watch.elapsed();
        Self {
            value,
            cost: 1, // TODO
            start_time,
            end_time,
        }
    }
}
