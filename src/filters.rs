// TODO: Move to `kurobako_filters` crate
use kurobako_core::filter::{Filter, FilterRecipe, FilterSpec};
use kurobako_core::num::FiniteF64;
use kurobako_core::solver::{ObservedObs, UnobservedObs};
use kurobako_core::{Error, Result};
use rand::distributions::{Distribution as _, Normal};
use rand::Rng;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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

    fn filter_ask<R: Rng>(&mut self, _rng: &mut R, obs: UnobservedObs) -> Result<UnobservedObs> {
        Ok(obs)
    }

    fn filter_tell<R: Rng>(&mut self, rng: &mut R, obs: ObservedObs) -> Result<ObservedObs> {
        if self.values_domain.is_empty() {
            self.values_domain = obs
                .value
                .iter()
                .map(|&v| track!(MinMax::new(v, v)).map_err(Error::from))
                .collect::<Result<Vec<_>>>()?;
            trace!("Initial values domain: {:?}", self.values_domain);
            return Ok(obs);
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
            let noised_value = track!(FiniteF64::new(Normal::new(value.get(), sd).sample(rng)))?;
            trace!(
                "Noised value: {} (original={})",
                noised_value.get(),
                value.get()
            );
            values.push(noised_value);
        }
        Ok(obs.map_value(|_| values))
    }
}
