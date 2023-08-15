use clap::Command;

use crate::result::{self, CliRunResult};

pub trait RunnerOptions {
  fn build_args(cmd: Command) -> Command;
}
pub trait Runner {
  const NAME: &'static str;
  const ABOUT: &'static str;
  type Options: RunnerOptions;
  fn run(&self) -> result::CliRunResult;
  fn new(options: Self::Options) -> Self;
}
