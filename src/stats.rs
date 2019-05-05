use crate::study::StudyRecord;
use crate::Name;
use kurobako_core::Result;
use rustats::num::NonNanF64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsSummary(Vec<OptimizerSummary>);
impl StatsSummary {
    pub fn new(stats: &Stats) -> Self {
        let mut map = BTreeMap::new();
        for p in &stats.0 {
            for o in &p.optimizers {
                if !map.contains_key(&o.optimizer) {
                    map.insert(
                        o.optimizer.clone(),
                        OptimizerSummary::new(o.optimizer.clone()),
                    );
                }
            }
        }

        for p in &stats.0 {
            let (worst, best) = p.min_max(|o| o.best_score.avg);
            for o in &p.optimizers {
                if o.best_score.avg == worst.best_score.avg {
                    map.get_mut(&o.optimizer).unwrap().best_score.worsts += 1;
                }
                if o.best_score.avg == best.best_score.avg {
                    map.get_mut(&o.optimizer).unwrap().best_score.bests += 1;
                }
            }

            let (worst, best) = p.min_max(|o| o.auc.avg);
            for o in &p.optimizers {
                if o.auc.avg == worst.auc.avg {
                    map.get_mut(&o.optimizer).unwrap().auc.worsts += 1;
                }
                if o.auc.avg == best.auc.avg {
                    map.get_mut(&o.optimizer).unwrap().auc.bests += 1;
                }
            }

            let (best, worst) = p.min_max(|o| o.latency.avg);
            for o in &p.optimizers {
                if o.latency.avg == worst.latency.avg {
                    map.get_mut(&o.optimizer).unwrap().latency.worsts += 1;
                }
                if o.latency.avg == best.latency.avg {
                    map.get_mut(&o.optimizer).unwrap().latency.bests += 1;
                }
            }
        }

        Self(map.into_iter().map(|(_, v)| v).collect())
    }

    pub fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "## Statistics Summary")?;
        writeln!(
            writer,
            "| optimizer | Best Score (o/x) | AUC (o/x) | Latency (o/x) |"
        )?;
        writeln!(
            writer,
            "|:----------|-----------------:|----------:|--------------:|"
        )?;
        for o in &self.0 {
            writeln!(
                writer,
                "| {} | {:03}/{:03} | {:03}/{:03} | {:03}/{:03} |",
                o.name.as_json(),
                o.best_score.bests,
                o.best_score.worsts,
                o.auc.bests,
                o.auc.worsts,
                o.latency.bests,
                o.latency.worsts
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizerSummary {
    pub name: Name,
    pub best_score: VictoryStats,
    pub auc: VictoryStats,
    pub latency: VictoryStats,
}
impl OptimizerSummary {
    fn new(name: Name) -> Self {
        Self {
            name,
            best_score: VictoryStats::default(),
            auc: VictoryStats::default(),
            latency: VictoryStats::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VictoryStats {
    pub bests: usize,
    pub worsts: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats(Vec<ProblemStats>);
impl Stats {
    pub fn new(studies: &[StudyRecord]) -> Self {
        let mut problems = BTreeMap::new();
        for s in studies {
            problems.entry(&s.problem).or_insert_with(Vec::new).push(s);
        }
        let problems = problems
            .into_iter()
            .map(|(problem, studies)| ProblemStats::new(problem, &studies))
            .collect();
        Self(problems)
    }

    pub fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "# Statistics")?;
        for p in &self.0 {
            p.write_markdown(&mut writer)?;
            writeln!(writer)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemStats {
    pub problem: Name,
    pub optimizers: Vec<OptimizerStats>,
}
impl ProblemStats {
    fn new(name: &Name, studies: &[&StudyRecord]) -> Self {
        let mut optimizers = BTreeMap::new();
        for s in studies {
            optimizers
                .entry(&s.optimizer)
                .or_insert_with(Vec::new)
                .push(*s);
        }
        let optimizers = optimizers
            .into_iter()
            .map(|(optimizer, studies)| OptimizerStats::new(optimizer, &studies))
            .collect();
        Self {
            problem: name.clone(),
            optimizers,
        }
    }

    fn min_max<F>(&self, f: F) -> (&OptimizerStats, &OptimizerStats)
    where
        F: Fn(&OptimizerStats) -> f64,
    {
        let min = self
            .optimizers
            .iter()
            .min_by_key(|o| NonNanF64::new(f(o)).unwrap_or_else(|e| panic!("{}", e)))
            .expect("TODO");
        let max = self
            .optimizers
            .iter()
            .max_by_key(|o| NonNanF64::new(f(o)).unwrap_or_else(|e| panic!("{}", e)))
            .expect("TODO");
        (min, max)
    }

    fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        writeln!(writer, "### Problem: {}", self.problem.as_json())?;
        writeln!(writer)?;
        writeln!(
            writer,
            "| Optimizer | Best Score (SD) | AUC (SD) | Latency |"
        )?;
        writeln!(
            writer,
            "|:----------|----------------:|---------:|-------------:|"
        )?;
        for o in &self.optimizers {
            o.write_markdown(&mut writer)?;
        }
        writeln!(writer)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizerStats {
    pub optimizer: Name,
    pub best_score: BasicStats,
    pub auc: BasicStats,
    pub latency: BasicStats,
}
impl OptimizerStats {
    fn new(name: &Name, studies: &[&StudyRecord]) -> Self {
        let best_scores = studies.iter().map(|s| s.best_score()).collect::<Vec<_>>();
        let aucs = studies.iter().map(|s| s.auc()).collect::<Vec<_>>();
        let latencies = studies
            .iter()
            .flat_map(|s| s.ack_latencies())
            .collect::<Vec<_>>();

        Self {
            optimizer: name.clone(),
            best_score: BasicStats::new(&best_scores),
            auc: BasicStats::new(&aucs),
            latency: BasicStats::new(&latencies),
        }
    }

    fn write_markdown<W: Write>(&self, mut writer: W) -> Result<()> {
        write!(writer, "| {} ", self.optimizer.as_json())?;
        write!(
            writer,
            "| {:.3} ({:.3}) ",
            self.best_score.avg, self.best_score.sd
        )?;
        write!(writer, "| {:.3} ({:.3}) ", self.auc.avg, self.auc.sd)?;
        write!(writer, "| {:.6} ", self.latency.avg)?;
        writeln!(writer, "|")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BasicStats {
    pub avg: f64,
    pub sd: f64,
}
impl BasicStats {
    fn new(xs: &[f64]) -> Self {
        let sum = xs.iter().sum::<f64>();
        let avg = sum / xs.len() as f64;
        let sd = (xs.iter().map(|&x| (x - avg).powi(2)).sum::<f64>() / xs.len() as f64).sqrt();
        Self { avg, sd }
    }
}
