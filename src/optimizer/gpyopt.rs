use super::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder, OptimizerBuilder};
use crate::{Error, ProblemSpace, Result};
use rand::Rng;
use std::fs;
use std::io::Write as _;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGenerator, Observation};
use yamakan::{self, Optimizer};

#[derive(Debug)]
pub struct GpyoptOptimizer {
    inner: ExternalCommandOptimizer,
    temp: TempPath,
}
impl Optimizer for GpyoptOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGenerator>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Observation<Self::Param, ()>> {
        track!(self.inner.ask(rng, idgen))
    }

    fn tell(&mut self, obs: Observation<Self::Param, Self::Value>) -> yamakan::Result<()> {
        track!(self.inner.tell(obs))
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct GpyoptOptimizerBuilder {}
impl OptimizerBuilder for GpyoptOptimizerBuilder {
    type Optimizer = GpyoptOptimizer;

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer> {
        let python_code = include_str!("../../contrib/optimizers/gpyopt_optimizer.py");
        let mut temp = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(temp.as_file_mut(), "{}", python_code).map_err(Error::from))?;

        let temp = temp.into_temp_path();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            track!(
                fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)).map_err(Error::from)
            )?;
        }

        let builder = ExternalCommandOptimizerBuilder {
            name: temp.to_path_buf(),
            args: vec![],
        };

        track!(builder.build(problem_space)).map(|inner| GpyoptOptimizer { inner, temp })
    }
}
