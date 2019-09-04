// TODO: Move to `kurobako_filters` crate
use kurobako_core::filter::{Filter, FilterRecipe, FilterSpec};
use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{self, ParamDomain, ParamValue};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{ObservedObs, UnobservedObs};
use kurobako_core::{Error, ErrorKind, Result};
use rand::distributions::Distribution as _;
use rand::Rng;
use rand_distr::Normal;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use structopt::StructOpt;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct GaussianNoiseFilterRecipe {
    #[structopt(long, default_value = "0.1")]
    level: f64,
}
impl FilterRecipe for GaussianNoiseFilterRecipe {
    type Filter = GaussianNoiseFilter;

    fn create_filter(&self) -> Result<Self::Filter> {
        Ok(GaussianNoiseFilter {
            level: self.level,
            values_domain: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub struct GaussianNoiseFilter {
    level: f64,

    // TODO: use (for example) 90%-tile instead of min-max
    values_domain: Vec<MinMax<FiniteF64>>, // observed
}
impl Filter for GaussianNoiseFilter {
    fn specification(&self) -> FilterSpec {
        FilterSpec {
            name: "gaussian-noise".to_owned(),
        }
    }

    fn filter_problem_spec(&mut self, _spec: &mut ProblemSpec) -> Result<()> {
        Ok(())
    }

    fn filter_ask<R: Rng>(&mut self, _rng: &mut R, _obs: &mut UnobservedObs) -> Result<()> {
        Ok(())
    }

    fn filter_tell<R: Rng>(&mut self, rng: &mut R, obs: &mut ObservedObs) -> Result<()> {
        if self.values_domain.is_empty() {
            self.values_domain = obs
                .value
                .iter()
                .map(|&v| track!(MinMax::new(v, v)).map_err(Error::from))
                .collect::<Result<Vec<_>>>()?;
            trace!("Initial values domain: {:?}", self.values_domain);
            return Ok(());
        }

        let mut values = Vec::with_capacity(obs.value.len());
        for (value, domain) in obs.value.iter().zip(self.values_domain.iter_mut()) {
            if value < domain.min() {
                *domain = track!(MinMax::new(*value, *domain.max()))?;
                trace!("Value domain updated: {:?}", domain);
            } else if value > domain.max() {
                *domain = track!(MinMax::new(*domain.min(), *value))?;
                trace!("Value domain updated: {:?}", domain);
            }

            let sd = domain.width().get() * self.level;
            let normal = Normal::new(value.get(), sd).unwrap_or_else(|e| panic!("TODO: {:?}", e));
            let noised_value = track!(FiniteF64::new(normal.sample(rng)))?;
            trace!(
                "Noised value: {} (original={})",
                noised_value.get(),
                value.get()
            );
            values.push(noised_value);
        }

        obs.value = values;
        Ok(())
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct DiscreteToContinuousFilterRecipe {}
impl FilterRecipe for DiscreteToContinuousFilterRecipe {
    type Filter = DiscreteToContinuousFilter;

    fn create_filter(&self) -> Result<Self::Filter> {
        Ok(DiscreteToContinuousFilter {
            indices: Vec::new(),
            originals: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub struct DiscreteToContinuousFilter {
    indices: Vec<usize>,
    originals: HashMap<ObsId, Vec<ParamValue>>,
}
impl Filter for DiscreteToContinuousFilter {
    fn specification(&self) -> FilterSpec {
        FilterSpec {
            name: "discrete-to-continuous".to_owned(),
        }
    }

    fn filter_problem_spec(&mut self, spec: &mut ProblemSpec) -> Result<()> {
        for (i, p) in spec.params_domain.iter_mut().enumerate() {
            if let ParamDomain::Discrete(d) = p {
                let new_p = track!(parameter::uniform(
                    &d.name,
                    d.range.low as f64,
                    d.range.high as f64
                ))?;
                *p = new_p;
                self.indices.push(i);
            }
        }

        Ok(())
    }

    fn filter_ask<R: Rng>(&mut self, _rng: &mut R, obs: &mut UnobservedObs) -> Result<()> {
        self.originals.insert(obs.id, obs.param.get().clone());
        for &i in &self.indices {
            track_assert!(i < obs.param.get().len(), ErrorKind::Bug; i, obs.param.get().len());

            let p = &mut obs.param.get_mut()[i];
            if let ParamValue::Continuous(c) = p {
                let new_p = ParamValue::Discrete(c.get() as i64);
                *p = new_p;
            } else {
                track_panic!(ErrorKind::Bug);
            }
        }
        Ok(())
    }

    fn filter_tell<R: Rng>(&mut self, _rng: &mut R, obs: &mut ObservedObs) -> Result<()> {
        let params = track_assert_some!(self.originals.remove(&obs.id), ErrorKind::Other);
        *obs.param.get_mut() = params;
        Ok(())
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct OneHotEncodingFilterRecipe {}
impl FilterRecipe for OneHotEncodingFilterRecipe {
    type Filter = OneHotEncodingFilter;

    fn create_filter(&self) -> Result<Self::Filter> {
        Ok(OneHotEncodingFilter {
            mappings: HashMap::new(),
            originals: HashMap::new(),
        })
    }
}

#[derive(Debug)]
pub struct OneHotEncodingFilter {
    mappings: HashMap<usize, usize>,
    originals: HashMap<ObsId, Vec<ParamValue>>,
}
impl Filter for OneHotEncodingFilter {
    fn specification(&self) -> FilterSpec {
        FilterSpec {
            name: "one-hot-encoding".to_owned(),
        }
    }

    fn filter_problem_spec(&mut self, spec: &mut ProblemSpec) -> Result<()> {
        let mut new_params_domain = Vec::new();
        for p in &spec.params_domain {
            if let ParamDomain::Categorical(d) = p {
                for j in 0..d.choices.len() {
                    let name = format!("{}_{}", d.name, j);
                    new_params_domain.push(track!(parameter::uniform(&name, 0.0, 1.0))?);
                }
                self.mappings
                    .insert(new_params_domain.len() - d.choices.len(), d.choices.len());
            } else {
                new_params_domain.push(p.clone());
            }
        }
        spec.params_domain = new_params_domain;

        Ok(())
    }

    fn filter_ask<R: Rng>(&mut self, _rng: &mut R, obs: &mut UnobservedObs) -> Result<()> {
        self.originals.insert(obs.id, obs.param.get().clone());
        let mut params = Vec::new();
        let mut i = 0;
        while i < obs.param.get().len() {
            if let Some(&n) = self.mappings.get(&i) {
                let selected = obs.param.get()[i..]
                    .iter()
                    .take(n)
                    .map(|p| FiniteF64::new(p.to_f64()).unwrap_or_else(|e| unreachable!("{}", e)))
                    .enumerate()
                    .map(|(idx, p)| (p, idx))
                    .max()
                    .unwrap_or_else(|| unreachable!())
                    .1;
                params.push(ParamValue::Categorical(selected));
                i += n;
            } else {
                params.push(obs.param.get()[i].clone());
                i += 1;
            }
        }
        *obs.param.get_mut() = params;

        Ok(())
    }

    fn filter_tell<R: Rng>(&mut self, _rng: &mut R, obs: &mut ObservedObs) -> Result<()> {
        let params = track_assert_some!(self.originals.remove(&obs.id), ErrorKind::Other);
        *obs.param.get_mut() = params;
        Ok(())
    }
}
