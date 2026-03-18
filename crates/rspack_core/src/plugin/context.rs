use rspack_util::fx_hash::FxDashMap;

use crate::{
  CompilationHooks, CompilerHooks, CompilerOptions, ConcatenatedModuleHooks,
  ContextModuleFactoryHooks, Generator, GeneratorOptions, ModuleType,
  NormalModuleFactoryHooks, NormalModuleHooks, Parser, ParserOptions,
};

pub type BoxedParser = Box<dyn Parser>;
pub type BoxedGenerator = Box<dyn Generator>;
pub type BoxedParserBuilder =
  Box<dyn 'static + Send + Sync + Fn(Option<&ParserOptions>) -> BoxedParser>;
pub type BoxedGeneratorBuilder =
  Box<dyn 'static + Send + Sync + Fn(Option<&GeneratorOptions>) -> BoxedGenerator>;

#[non_exhaustive]
pub struct ApplyContext<'c> {
  pub(crate) registered_parser_builder: &'c mut FxDashMap<ModuleType, BoxedParserBuilder>,
  pub(crate) registered_generator_builder: &'c mut FxDashMap<ModuleType, BoxedGeneratorBuilder>,
  pub compiler_hooks: &'c mut CompilerHooks,
  pub compilation_hooks: &'c mut CompilationHooks,
  pub normal_module_factory_hooks: &'c mut NormalModuleFactoryHooks,
  pub context_module_factory_hooks: &'c mut ContextModuleFactoryHooks,
  pub normal_module_hooks: &'c mut NormalModuleHooks,
  pub concatenated_module_hooks: &'c mut ConcatenatedModuleHooks,

  pub compiler_options: &'c CompilerOptions,
}

impl ApplyContext<'_> {
  pub fn register_parser_builder(
    &mut self,
    module_type: ModuleType,
    parser_builder: BoxedParserBuilder,
  ) {
    self
      .registered_parser_builder
      .insert(module_type, parser_builder);
  }

  pub fn register_generator_builder(
    &mut self,
    module_type: ModuleType,
    generator_builder: BoxedGeneratorBuilder,
  ) {
    self
      .registered_generator_builder
      .insert(module_type, generator_builder);
  }
}
