#[macro_use]
extern crate trackable;

use kurobako::markdown::MarkdownWriter;
use kurobako::plot::PlotOpt;
use kurobako::problem::KurobakoProblemRecipe;
use kurobako::report::{ReportOpt, Reporter};
use kurobako::runner::{Runner, RunnerOpt};
use kurobako::solver::KurobakoSolverRecipe;
use kurobako::study::{StudiesRecipe, StudyRecipe};
use kurobako_core::json;
use kurobako_core::Error;
use std::io;
use structopt::StructOpt;

// use kurobako::exam::ExamRecipe;
// use kurobako::multi_exam::MultiExamRecipe;
// use kurobako::plot::PlotOptions;
// use kurobako::plot_scatter::PlotScatterOptions;
// use kurobako::problem_suites::{KurobakoProblemSuite, ProblemSuite};
// use kurobako::record::{BenchmarkRecord, StudyRecord};
// use kurobako::variable::Variable;
// use kurobako_core::{Error, Result};
// use std::path::PathBuf;

macro_rules! print_json {
    ($x:expr) => {
        track!(serde_json::to_writer(std::io::stdout().lock(), &$x).map_err(Error::from))?;
        println!();
    };
    ($x:expr, $out:expr) => {
        track!(serde_json::to_writer(&mut $out, &$x).map_err(Error::from))?;
        println!();
    };
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Solver(KurobakoSolverRecipe),
    Problem(KurobakoProblemRecipe),
    Study(StudyRecipe),
    Studies(StudiesRecipe),
    Run(RunnerOpt),
    Report(ReportOpt),
    Plot(PlotOpt),
    // ProblemSuite(KurobakoProblemSuite),
    // Exam(ExamRecipe),
    // MultiExam(MultiExamRecipe),
    // Var(Variable),
}

fn main() -> trackable::result::TopLevelResult {
    let opt = Opt::from_args();

    match opt {
        Opt::Solver(x) => {
            print_json!(x);
        }
        Opt::Problem(x) => {
            print_json!(x);
        }
        Opt::Study(x) => {
            print_json!(x);
        }
        Opt::Studies(x) => {
            for y in x.studies() {
                print_json!(y);
            }
        }
        Opt::Run(opt) => {
            track!(Runner::new(opt).run())?;
        }
        Opt::Report(opt) => {
            let studies = track!(json::load(io::stdin().lock()))?;
            let reporter = Reporter::new(studies, opt);
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            let mut writer = MarkdownWriter::new(&mut stdout);
            track!(reporter.report_all(&mut writer))?;
        }
        Opt::Plot(opt) => {
            let studies = track!(json::load(io::stdin().lock()))?;
            track!(opt.plot(&studies))?;
        }
    }

    Ok(())
}

//         Opt::Exam(p) => {
//             track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
//         }
//         Opt::MultiExam(p) => {
//             track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
//         }
//         Opt::Var(p) => {
//             track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
//         }
//         Opt::ProblemSuite(p) => {
//             for p in p.problem_specs() {
//                 track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?;
//                 println!();
//             }
//         }
//     }
//     Ok(())
// }

// fn handle_plot_command(opt: PlotOpt) -> Result<()> {
//     use std::fs;

//     track!(fs::create_dir_all(&opt.output_dir).map_err(Error::from); opt.output_dir)?;

//     let stdin = std::io::stdin();
//     let mut studies = Vec::new();
//     for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
//         match track!(study.map_err(Error::from)) {
//             Err(e) => {
//                 eprintln!("{}", e);
//             }
//             Ok(study) => {
//                 let study: StudyRecord = study;
//                 // TODO
//                 // if let Some(budget) = opt.budget {
//                 //     study.limit_budget(budget);
//                 // }
//                 studies.push(study);
//             }
//         }
//     }

//     track!(opt.inner.plot_problems(&studies, opt.output_dir))?;
//     Ok(())
// }

// fn handle_plot_scatter_command(opt: PlotScatterOpt) -> Result<()> {
//     use std::fs;

//     track!(fs::create_dir_all(&opt.output_dir).map_err(Error::from); opt.output_dir)?;

//     let stdin = std::io::stdin();
//     let mut studies = Vec::new();
//     for study in serde_json::Deserializer::from_reader(stdin.lock()).into_iter() {
//         match track!(study.map_err(Error::from)) {
//             Err(e) => {
//                 eprintln!("{}", e);
//             }
//             Ok(study) => {
//                 let study: StudyRecord = study;
//                 // TODO
//                 // if let Some(budget) = opt.budget {
//                 //     study.limit_budget(budget);
//                 // }
//                 studies.push(study);
//             }
//         }
//     }

//     track!(opt.inner.plot_problems(&studies, opt.output_dir))?;
//     Ok(())
// }
