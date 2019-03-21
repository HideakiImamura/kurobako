#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

pub use self::problem::{Evaluate, Problem, ProblemSpace, ProblemSpec};

pub mod distribution;
pub mod optimizer;
pub mod problems;
pub mod runner;
pub mod study;
pub mod summary;
pub mod time;
pub mod trial;

mod float;
mod problem;
