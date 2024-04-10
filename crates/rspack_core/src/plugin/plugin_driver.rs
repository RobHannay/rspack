use std::sync::{Arc, Mutex};

use derivative::Derivative;
use rspack_error::{Diagnostic, Result};
use rspack_util::fx_hash::FxDashMap;
use tracing::instrument;

use crate::{
  AdditionalChunkRuntimeRequirementsArgs, AdditionalModuleRequirementsArgs, ApplyContext,
  BoxedParserAndGeneratorBuilder, Compilation, CompilationHooks, CompilerHooks, CompilerOptions,
  ContextModuleFactoryHooks, ModuleIdentifier, ModuleType, NormalModuleFactoryHooks,
  NormalModuleHooks, OptimizeChunksArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginAdditionalModuleRequirementsOutput, PluginContext, PluginRuntimeRequirementsInTreeOutput,
  ResolverFactory, RuntimeRequirementsInTreeArgs,
};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PluginDriver {
  pub(crate) options: Arc<CompilerOptions>,
  pub plugins: Vec<Box<dyn Plugin>>,
  pub resolver_factory: Arc<ResolverFactory>,
  #[derivative(Debug = "ignore")]
  pub registered_parser_and_generator_builder:
    FxDashMap<ModuleType, BoxedParserAndGeneratorBuilder>,
  /// Collecting error generated by plugin phase, e.g., `Syntax Error`
  pub diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
  pub compiler_hooks: CompilerHooks,
  pub compilation_hooks: CompilationHooks,
  pub normal_module_factory_hooks: NormalModuleFactoryHooks,
  pub context_module_factory_hooks: ContextModuleFactoryHooks,
  pub normal_module_hooks: NormalModuleHooks,
}

impl PluginDriver {
  pub fn new(
    mut options: CompilerOptions,
    plugins: Vec<Box<dyn Plugin>>,
    resolver_factory: Arc<ResolverFactory>,
  ) -> (Arc<Self>, Arc<CompilerOptions>) {
    let mut compiler_hooks = Default::default();
    let mut compilation_hooks = Default::default();
    let mut normal_module_factory_hooks = Default::default();
    let mut context_module_factory_hooks = Default::default();
    let mut normal_module_hooks = Default::default();
    let mut registered_parser_and_generator_builder = FxDashMap::default();
    let mut apply_context = ApplyContext {
      registered_parser_and_generator_builder: &mut registered_parser_and_generator_builder,
      compiler_hooks: &mut compiler_hooks,
      compilation_hooks: &mut compilation_hooks,
      normal_module_factory_hooks: &mut normal_module_factory_hooks,
      context_module_factory_hooks: &mut context_module_factory_hooks,
      normal_module_hooks: &mut normal_module_hooks,
    };
    for plugin in &plugins {
      plugin
        .apply(
          PluginContext::with_context(&mut apply_context),
          &mut options,
        )
        .expect("TODO:");
    }

    let options = Arc::new(options);

    (
      Arc::new(Self {
        options: options.clone(),
        plugins,
        resolver_factory,
        registered_parser_and_generator_builder,
        diagnostics: Arc::new(Mutex::new(vec![])),
        compiler_hooks,
        compilation_hooks,
        normal_module_factory_hooks,
        context_module_factory_hooks,
        normal_module_hooks,
      }),
      options,
    )
  }

  pub fn take_diagnostic(&self) -> Vec<Diagnostic> {
    let mut diagnostic = self.diagnostics.lock().expect("TODO:");
    std::mem::take(&mut diagnostic)
  }

  #[instrument(name = "plugin:module_asset", skip_all)]
  pub async fn module_asset(&self, module: ModuleIdentifier, asset_name: String) -> Result<()> {
    for plugin in &self.plugins {
      plugin.module_asset(module, asset_name.clone()).await?;
    }

    Ok(())
  }

  #[instrument(name = "plugin:additional_chunk_runtime_requirements", skip_all)]
  pub async fn additional_chunk_runtime_requirements(
    &self,
    args: &mut AdditionalChunkRuntimeRequirementsArgs<'_>,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    for plugin in &self.plugins {
      plugin
        .additional_chunk_runtime_requirements(PluginContext::new(), args)
        .await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:additional_tree_runtime_requirements", skip_all)]
  pub async fn additional_tree_runtime_requirements(
    &self,
    args: &mut AdditionalChunkRuntimeRequirementsArgs<'_>,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    for plugin in &self.plugins {
      plugin
        .additional_tree_runtime_requirements(PluginContext::new(), args)
        .await?;
    }
    Ok(())
  }

  pub fn runtime_requirement_in_module(
    &self,
    args: &mut AdditionalModuleRequirementsArgs,
  ) -> PluginAdditionalModuleRequirementsOutput {
    for plugin in &self.plugins {
      plugin.runtime_requirements_in_module(PluginContext::new(), args)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:runtime_requirements_in_tree", skip_all)]
  pub async fn runtime_requirements_in_tree(
    &self,
    args: &mut RuntimeRequirementsInTreeArgs<'_>,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    for plugin in &self.plugins {
      plugin
        .runtime_requirements_in_tree(PluginContext::new(), args)
        .await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:optimize_chunks", skip_all)]
  pub async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &self.plugins {
      plugin
        .optimize_chunks(PluginContext::new(), OptimizeChunksArgs { compilation })
        .await?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:optimize_dependencies", skip_all)]
  pub async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<()>> {
    for plugin in &self.plugins {
      if let Some(t) = plugin.optimize_dependencies(compilation).await? {
        return Ok(Some(t));
      };
    }
    Ok(None)
  }

  #[instrument(name = "plugin:optimize_code_generation", skip_all)]
  pub async fn optimize_code_generation(
    &self,
    compilation: &mut Compilation,
  ) -> Result<Option<()>> {
    for plugin in &self.plugins {
      if let Some(t) = plugin.optimize_code_generation(compilation).await? {
        return Ok(Some(t));
      };
    }
    Ok(None)
  }

  #[instrument(name = "plugin:module_ids", skip_all)]
  pub fn module_ids(&self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &self.plugins {
      plugin.module_ids(compilation)?;
    }
    Ok(())
  }

  #[instrument(name = "plugin:chunk_ids", skip_all)]
  pub fn chunk_ids(&self, compilation: &mut Compilation) -> Result<()> {
    for plugin in &self.plugins {
      plugin.chunk_ids(compilation)?;
    }
    Ok(())
  }
}
