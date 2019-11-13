//! Domain of parameter and objective values.
use crate::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std;

/// Domain.
///
/// A `Domain` instance consists of a vector of `Variable`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain(Vec<Variable>);
impl Domain {
    /// Makes a new `Domain` instance.
    pub fn new(variables: Vec<VariableBuilder>) -> Result<Self> {
        let mut vars = Vec::<Variable>::new();
        for v in variables.into_iter() {
            let v = track!(v.finish())?;

            track_assert!(
                vars.iter().all(|var| v.name != var.name),
                ErrorKind::InvalidInput,
                "Duplicate name: {:?}",
                v.name
            );

            for c in &v.conditions {
                track!(c.validate(&vars))?;
            }

            vars.push(v);
        }
        Ok(Self(vars))
    }

    /// Returns a reference to the variables in this domain.
    pub fn variables(&self) -> &[Variable] {
        &self.0
    }
}

/// Returns a `VariableBuilder` which was initialized with the given variable name.
///
/// This is equivalent to `VariableBuilder::new(name)`.
pub fn var(name: &str) -> VariableBuilder {
    VariableBuilder::new(name)
}

/// `Variable` builder.
#[derive(Debug)]
pub struct VariableBuilder {
    name: String,
    range: Range,
    distribution: Distribution,
    conditions: Vec<Condition>,
}
impl VariableBuilder {
    /// Makes a new `VariableBuilder` with the given variable name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            range: Range::Continuous {
                low: std::f64::NEG_INFINITY,
                high: std::f64::INFINITY,
            },
            distribution: Distribution::Uniform,
            conditions: Vec::new(),
        }
    }

    /// Sets the distribution of this variable to `Distribution::Uniform`.
    ///
    /// Note that `Distribution::Uniform` is the default distribution.
    pub fn uniform(mut self) -> Self {
        self.distribution = Distribution::Uniform;
        self
    }

    /// Sets the distribution of this variable to `Distribution::LogUniform`.
    pub fn log_uniform(mut self) -> Self {
        self.distribution = Distribution::LogUniform;
        self
    }

    /// Sets the range of this variable to the given continuous numerical range.
    pub fn continuous(mut self, low: f64, high: f64) -> Self {
        self.range = Range::Continuous { low, high };
        self
    }

    /// Sets the range of this variable to the given discrete numerical range.
    pub fn discrete(mut self, low: i64, high: i64) -> Self {
        self.range = Range::Discrete { low, high };
        self
    }

    /// Sets the range of this variable to the given categorical range.
    pub fn categorical<I, T>(mut self, choices: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        self.range = Range::Categorical {
            choices: choices.into_iter().map(|c| c.as_ref().to_owned()).collect(),
        };
        self
    }

    /// Sets the range of this variable to boolean.
    ///
    /// This is equivalent to `self.categorical(&["false", "true"])`.
    pub fn boolean(self) -> Self {
        self.categorical(&["false", "true"])
    }

    /// Adds an evaluation condition to this variable.
    pub fn condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Builds a `Variable` instance with the given settings.
    pub fn finish(self) -> Result<Variable> {
        match &self.range {
            Range::Continuous { low, high } => {
                track_assert!(low < high, ErrorKind::InvalidInput; self)
            }
            Range::Discrete { low, high } => {
                track_assert!(low < high, ErrorKind::InvalidInput; self)
            }
            Range::Categorical { choices } => {
                track_assert!(choices.len() > 0, ErrorKind::InvalidInput; self)
            }
        }

        if self.distribution == Distribution::LogUniform {
            match self.range {
                Range::Continuous { low, .. } if 0.0 < low => {}
                Range::Discrete { low, .. } if 0 < low => {}
                _ => track_panic!(ErrorKind::InvalidInput; self),
            }
        }

        Ok(Variable {
            name: self.name,
            range: self.range,
            distribution: self.distribution,
            conditions: self.conditions,
        })
    }
}

/// A variable in a domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    name: String,
    range: Range,
    distribution: Distribution,
    conditions: Vec<Condition>,
}
impl Variable {
    /// Returns the name of this variable.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the value range of this variable.
    pub fn range(&self) -> &Range {
        &self.range
    }

    /// Returns the prior distribution of the value of this variable.
    pub fn distribution(&self) -> Distribution {
        self.distribution
    }

    /// Returns the conditions required to evaluate this variable.
    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }
}

/// Distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Distribution {
    Uniform,
    LogUniform,
}

/// Variable range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Range {
    /// Continuous numerical range: `[low..high)`.
    Continuous { low: f64, high: f64 },

    /// Discrete numerical range: `[low..high)`.
    Discrete { low: i64, high: i64 },

    /// Categorical range.
    Categorical { choices: Vec<String> },
}
impl Range {
    fn contains(&self, v: f64) -> bool {
        match self {
            Self::Continuous { low, high } => *low <= v && v < *high,
            Self::Discrete { low, high } => *low as f64 <= v && v < *high as f64,
            Self::Categorical { choices } => 0.0 <= v && v < choices.len() as f64,
        }
    }
}
impl Eq for Range {}

/// Evaluation condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Condition {
    /// This condition holds if the value of the variable named `target` is equal to `value`.
    Eq { target: String, value: f64 },
}
impl Condition {
    fn validate(&self, preceding_variables: &[Variable]) -> Result<()> {
        let Condition::Eq { target, value } = self;

        for v in preceding_variables {
            if target != &v.name {
                continue;
            }

            track_assert!(v.range.contains(*value), ErrorKind::InvalidInput; self);
        }

        track_panic!(ErrorKind::InvalidInput; self);
    }
}
impl Eq for Condition {}