mod context_dependency_helper;
mod parser;
mod util;

use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, BuildInfo, BuildMeta,
  CompilerOptions, FactoryMeta, JavascriptParserOptions, ModuleIdentifier, ModuleLayer, ModuleType,
  ParseMeta, ResourceData, SideEffectsBailoutItemWithSpan,
};
use rspack_error::Diagnostic;
use rspack_javascript_compiler::ast::Program;
use rustc_hash::FxHashSet;
use swc_core::common::{BytePos, Mark, comments::Comments};

pub use self::{
  context_dependency_helper::{ContextModuleScanResult, create_context_dependency},
  parser::{
    AllowedMemberTypes, AtomMembers, CallExpressionInfo, CallHooksName,
    DestructuringAssignmentProperties, DestructuringAssignmentProperty, ExportedVariableInfo,
    JavascriptParserState, MemberExpressionInfo, MemberRanges, OptionalMembers, RootName,
    ScopeTerminated, TagInfoData, TopLevelScope, ast::*, estree::*,
  },
  util::*,
};
use crate::parser_and_generator::{JavaScriptParser, ParserRuntimeRequirementsData};

pub struct ScanDependenciesResult {
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub warning_diagnostics: Vec<Diagnostic>,
  pub side_effects_item: Option<SideEffectsBailoutItemWithSpan>,
}

#[allow(clippy::too_many_arguments)]
pub fn scan_dependencies(
  source: &str,
  program: &Program,
  resource_data: &ResourceData,
  compiler_options: &CompilerOptions,
  javascript_options: &JavascriptParserOptions,
  module_type: &ModuleType,
  module_layer: Option<&ModuleLayer>,
  factory_meta: Option<&FactoryMeta>,
  build_meta: &mut BuildMeta,
  build_info: &mut BuildInfo,
  module_identifier: ModuleIdentifier,
  semicolons: &mut FxHashSet<BytePos>,
  unresolved_mark: Mark,
  parser: &JavaScriptParser,
  parse_meta: ParseMeta,
  parser_runtime_requirements: &ParserRuntimeRequirementsData,
) -> Result<ScanDependenciesResult, Vec<Diagnostic>> {
  let hooks = parser.get_or_init_hooks(
    compiler_options,
    javascript_options,
    module_type,
    parser_runtime_requirements,
  );
  let mut parser = JavascriptParserState::new(
    source,
    compiler_options,
    javascript_options,
    program.comments.as_ref().map(|c| c as &dyn Comments),
    &module_identifier,
    module_type,
    module_layer,
    resource_data,
    factory_meta,
    build_meta,
    build_info,
    semicolons,
    parse_meta,
    parser_runtime_requirements,
    hooks,
    unresolved_mark,
  );

  parser.walk_program(program.get_inner_program());
  parser.into_results()
}
