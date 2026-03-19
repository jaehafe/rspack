use std::{
  borrow::Cow,
  sync::{Arc, LazyLock, OnceLock},
};

use regex::Regex;
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BuildMetaExportsType, COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY,
  ChunkGraph, CollectedTypeScriptInfo, Compilation, CompilerOptions, DependenciesBlock,
  DependencyId, GenerateContext, Generator, ImportMeta, JavascriptParserCommonjsExportsOption,
  JavascriptParserOptions, Module, ModuleCodeTemplate, ModuleGraph, ModuleType, ParseContext,
  ParseResult, Parser, RuntimeGlobals, SideEffectsBailoutItem, SourceType, TemplateContext,
  TemplateReplaceSource,
  diagnostics::map_box_diagnostics_to_module_parse_diagnostics,
  remove_bom, render_init_fragments,
  rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use rspack_javascript_compiler::JavaScriptCompiler;
use swc_core::{
  base::config::IsModule,
  common::{BytePos, input::SourceFileInput},
  ecma::{
    ast,
    parser::{EsSyntax, Syntax, lexer::Lexer},
    transforms::base::fixer::paren_remover,
  },
};
use swc_node_comments::SwcComments;

use crate::{
  BoxJavascriptParserPlugin,
  dependency::ESMCompatibilityDependency,
  parser_plugin::{
    self, JavascriptParserHooks, JavascriptParserPlugin, JavascriptParserPluginContext,
  },
  visitors::{ScanDependenciesResult, scan_dependencies, semicolon, swc_visitor::resolver},
};

fn module_type_to_is_module(value: &ModuleType) -> IsModule {
  // parser options align with webpack
  match value {
    ModuleType::JsEsm => IsModule::Bool(true),
    ModuleType::JsDynamic => IsModule::Bool(false),
    _ => IsModule::Unknown,
  }
}

#[derive(Debug)]
pub struct ParserRuntimeRequirementsData {
  pub module: String,
  pub exports: String,
  pub require: String,
  pub require_regex: &'static LazyLock<Regex>,
}

static LEGACY_REQUIRE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new("__webpack_require__\\s*(!?\\.)").expect("should init `REQUIRE_FUNCTION_REGEX`")
});

impl ParserRuntimeRequirementsData {
  pub fn new(runtime_template: &ModuleCodeTemplate) -> Self {
    let require_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::REQUIRE);
    let module_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::MODULE);
    let exports_name =
      runtime_template.render_runtime_globals_without_adding(&RuntimeGlobals::EXPORTS);
    Self {
      require_regex: &LEGACY_REQUIRE_REGEX,
      module: module_name,
      exports: exports_name,
      require: require_name,
    }
  }
}

#[cacheable]
#[derive(Default)]
pub struct JavaScriptParser {
  // TODO
  #[cacheable(with=Skip)]
  parser_plugins: Vec<BoxJavascriptParserPlugin>,
  #[cacheable(with=Skip)]
  hooks: OnceLock<Arc<JavascriptParserHooks>>,
}

impl std::fmt::Debug for JavaScriptParser {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("JavaScriptParser")
      .field("parser_plugins", &"...")
      .finish()
  }
}

impl JavaScriptParser {
  pub fn add_parser_plugin(&mut self, parser_plugin: BoxJavascriptParserPlugin) {
    assert!(
      self.hooks.get().is_none(),
      "cannot add parser plugin after hooks initialization"
    );
    self.parser_plugins.push(parser_plugin);
  }

  pub(crate) fn get_or_init_hooks(
    &self,
    compiler_options: &CompilerOptions,
    javascript_options: &JavascriptParserOptions,
    module_type: &ModuleType,
    parser_runtime_requirements: &ParserRuntimeRequirementsData,
  ) -> Arc<JavascriptParserHooks> {
    self
      .hooks
      .get_or_init(|| {
        self.create_hooks(
          compiler_options,
          javascript_options,
          module_type,
          parser_runtime_requirements,
        )
      })
      .clone()
  }

  fn create_hooks(
    &self,
    compiler_options: &CompilerOptions,
    javascript_options: &JavascriptParserOptions,
    module_type: &ModuleType,
    parser_runtime_requirements: &ParserRuntimeRequirementsData,
  ) -> Arc<JavascriptParserHooks> {
    let mut hooks = JavascriptParserHooks::default();
    let mut context = JavascriptParserPluginContext {
      hooks: &mut hooks,
      parser_runtime_requirements,
    };

    for plugin in self.parser_plugins.iter().cloned() {
      plugin.apply(&mut context);
    }

    Arc::new(parser_plugin::InitializeEvaluating).apply(&mut context);
    Arc::new(parser_plugin::JavascriptMetaInfoPlugin).apply(&mut context);
    Arc::new(parser_plugin::CheckVarDeclaratorIdent).apply(&mut context);
    Arc::new(parser_plugin::ConstPlugin).apply(&mut context);
    Arc::new(parser_plugin::UseStrictPlugin).apply(&mut context);
    Arc::new(parser_plugin::RequireContextDependencyParserPlugin).apply(&mut context);
    Arc::new(parser_plugin::RequireEnsureDependenciesBlockParserPlugin).apply(&mut context);
    Arc::new(parser_plugin::CompatibilityPlugin).apply(&mut context);

    if module_type.is_js_auto() || module_type.is_js_esm() {
      Arc::new(parser_plugin::ESMTopLevelThisParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::ESMDetectionParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::ImportMetaContextDependencyParserPlugin).apply(&mut context);
      if matches!(
        javascript_options.import_meta,
        Some(ImportMeta::Enabled | ImportMeta::PreserveUnknown)
      ) {
        Arc::new(parser_plugin::ImportMetaPlugin(
          javascript_options.import_meta.expect("should have value"),
        ))
        .apply(&mut context);
      } else {
        Arc::new(parser_plugin::ImportMetaDisabledPlugin).apply(&mut context);
      }

      Arc::new(parser_plugin::ESMImportDependencyParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::ESMExportDependencyParserPlugin).apply(&mut context);
    }

    if compiler_options.amd.is_some() && (module_type.is_js_auto() || module_type.is_js_dynamic()) {
      Arc::new(parser_plugin::AMDRequireDependenciesBlockParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::AMDDefineDependencyParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::AMDParserPlugin).apply(&mut context);
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() {
      Arc::new(parser_plugin::CommonJsImportsParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::CommonJsPlugin).apply(&mut context);
      let commonjs_exports = javascript_options
        .commonjs
        .as_ref()
        .map_or(JavascriptParserCommonjsExportsOption::Enable, |commonjs| {
          commonjs.exports
        });
      if commonjs_exports != JavascriptParserCommonjsExportsOption::Disable {
        Arc::new(parser_plugin::CommonJsExportsParserPlugin::new(
          commonjs_exports == JavascriptParserCommonjsExportsOption::SkipInEsm,
        ))
        .apply(&mut context);
      }
    }

    let handle_cjs =
      (module_type.is_js_auto() || module_type.is_js_dynamic()) && compiler_options.node.is_some();
    let handle_esm = module_type.is_js_auto() || module_type.is_js_esm();
    if handle_cjs || handle_esm {
      Arc::new(parser_plugin::NodeStuffPlugin::new(handle_cjs, handle_esm)).apply(&mut context);
    }

    if module_type.is_js_auto() || module_type.is_js_dynamic() || module_type.is_js_esm() {
      Arc::new(parser_plugin::IsIncludedPlugin).apply(&mut context);
      Arc::new(parser_plugin::ExportsInfoApiPlugin).apply(&mut context);
      Arc::new(parser_plugin::APIPlugin::new(
        compiler_options.output.module,
      ))
      .apply(&mut context);
      Arc::new(parser_plugin::ImportParserPlugin).apply(&mut context);
      Arc::new(parser_plugin::WorkerPlugin::new(
        javascript_options
          .worker
          .as_ref()
          .expect("should have worker"),
      ))
      .apply(&mut context);
      Arc::new(parser_plugin::OverrideStrictPlugin).apply(&mut context);
    }

    if compiler_options.optimization.inline_exports {
      Arc::new(parser_plugin::InlineConstPlugin).apply(&mut context);
    }
    if compiler_options.optimization.inner_graph {
      Arc::new(parser_plugin::InnerGraphPlugin).apply(&mut context);
    }

    if compiler_options.optimization.side_effects.is_true() {
      Arc::new(parser_plugin::SideEffectsParserPlugin).apply(&mut context);
    }

    Arc::new(hooks)
  }
}

#[cacheable]
#[derive(Debug, Default)]
pub struct JavaScriptGenerator;

impl JavaScriptGenerator {
  fn source_block(
    &self,
    compilation: &Compilation,
    block_id: &AsyncDependenciesBlockIdentifier,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let module_graph = compilation.get_module_graph();
    let block = module_graph
      .block_by_id(block_id)
      .expect("should have block");
    //    let block = block_id.expect_get(compilation);
    block.get_dependencies().iter().for_each(|dependency_id| {
      self.source_dependency(compilation, dependency_id, source, context)
    });
    block
      .get_blocks()
      .iter()
      .for_each(|block_id| self.source_block(compilation, block_id, source, context));
  }

  fn source_dependency(
    &self,
    compilation: &Compilation,
    dependency_id: &DependencyId,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    if let Some(dependency) = compilation
      .get_module_graph()
      .dependency_by_id(dependency_id)
      .as_dependency_code_generation()
    {
      if let Some(template) = dependency
        .dependency_template()
        .and_then(|template_type| compilation.get_dependency_template(template_type))
      {
        template.render(dependency, source, context)
      } else {
        panic!(
          "Can not find dependency template of {:?}",
          dependency.dependency_template()
        );
      }
    }
  }
}

static SOURCE_TYPES: &[SourceType; 1] = &[SourceType::JavaScript];

#[cacheable_dyn]
#[async_trait::async_trait]
impl Parser for JavaScriptParser {
  #[tracing::instrument("JavaScriptParser:parse", skip_all,fields(
    resource = parse_context.resource_data.resource()
  ))]
  async fn parse<'a>(
    &self,
    parse_context: ParseContext<'a>,
  ) -> Result<TWithDiagnosticArray<ParseResult>> {
    let ParseContext {
      source,
      module_type,
      module_layer,
      resource_data,
      compiler_options,
      runtime_template,
      factory_meta,
      build_info,
      build_meta,
      module_identifier,
      loaders,
      module_parser_options,
      mut parse_meta,
      ..
    } = parse_context;
    let mut diagnostics: Vec<Diagnostic> = vec![];

    if let Some(collected_ts_info) = parse_meta.remove(COLLECTED_TYPESCRIPT_INFO_PARSE_META_KEY)
      && let Ok(collected_ts_info) =
        (collected_ts_info as Box<dyn std::any::Any>).downcast::<CollectedTypeScriptInfo>()
    {
      build_info.collected_typescript_info = Some(*collected_ts_info);
    }

    let default_with_diagnostics = |source: Arc<dyn Source>, diagnostics: Vec<Diagnostic>| {
      Ok(
        ParseResult {
          source,
          dependencies: vec![],
          blocks: vec![],
          presentational_dependencies: vec![],
          code_generation_dependencies: vec![],
          parser_data: None,
          side_effects_bailout: None,
        }
        .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
          diagnostics,
          loaders,
        )),
      )
    };

    let source = remove_bom(source);
    let source_string = source.source().into_string_lossy();

    let comments = SwcComments::default();
    let target = ast::EsVersion::EsNext;

    let javascript_options = module_parser_options
      .and_then(|options| options.get_javascript())
      .expect("should at least have a global javascript parser options");
    let jsx = javascript_options.jsx.unwrap_or(false);

    let parser_lexer = Lexer::new(
      Syntax::Es(EsSyntax {
        jsx,
        allow_return_outside_function: matches!(
          module_type,
          ModuleType::JsDynamic | ModuleType::JsAuto
        ),
        explicit_resource_management: true,
        import_attributes: true,
        ..Default::default()
      }),
      target,
      SourceFileInput::new(
        &source_string,
        BytePos(1),
        BytePos(source_string.len() as u32 + 1),
      ),
      Some(&comments),
    );

    let javascript_compiler = JavaScriptCompiler::new();

    let (mut ast, tokens) = match javascript_compiler.parse_with_lexer(
      &source_string,
      parser_lexer,
      module_type_to_is_module(module_type),
      Some(comments.clone()),
      true,
    ) {
      Ok(ast) => ast,
      Err(e) => {
        diagnostics.append(&mut e.into_inner().into_iter().map(|e| e.into()).collect());
        return default_with_diagnostics(source, diagnostics);
      }
    };

    let mut semicolons = Default::default();
    ast.transform(|program, context| {
      program.visit_mut_with(&mut paren_remover(Some(&comments)));
      program.visit_mut_with(&mut resolver(
        context.unresolved_mark,
        context.top_level_mark,
        false,
      ));
      program.visit_with(&mut semicolon::InsertedSemicolons {
        semicolons: &mut semicolons,
        // safety: it's safe to assert tokens is some since we pass with_tokens = true
        tokens: &tokens.expect("should get tokens from parser"),
      });
    });

    let unresolved_mark = ast.get_context().unresolved_mark;
    let parser_runtime_requirements = ParserRuntimeRequirementsData::new(runtime_template);

    if compiler_options.optimization.inline_exports {
      build_info.inline_exports = true;
    }

    let ScanDependenciesResult {
      dependencies,
      blocks,
      presentational_dependencies,
      mut warning_diagnostics,
      mut side_effects_item,
    } = match ast.visit(|program, _| {
      scan_dependencies(
        &source_string,
        program,
        resource_data,
        compiler_options,
        javascript_options,
        module_type,
        module_layer,
        factory_meta,
        build_meta,
        build_info,
        module_identifier,
        &mut semicolons,
        unresolved_mark,
        self,
        parse_meta,
        &parser_runtime_requirements,
      )
    }) {
      Ok(result) => result,
      Err(mut e) => {
        diagnostics.append(&mut e);
        return default_with_diagnostics(source, diagnostics);
      }
    };
    diagnostics.append(&mut warning_diagnostics);
    let mut side_effects_bailout = None;

    if compiler_options.optimization.side_effects.is_true() {
      build_meta.side_effect_free = Some(side_effects_item.is_none());
      side_effects_bailout = side_effects_item.take().and_then(|item| -> Option<_> {
        let msg = item.loc?.to_string();
        Some(SideEffectsBailoutItem { msg, ty: item.ty })
      });
    }

    Ok(
      ParseResult {
        source,
        dependencies,
        blocks,
        presentational_dependencies,
        code_generation_dependencies: vec![],
        parser_data: None,
        side_effects_bailout,
      }
      .with_diagnostic(map_box_diagnostics_to_module_parse_diagnostics(
        diagnostics,
        loaders,
      )),
    )
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Generator for JavaScriptGenerator {
  fn source_types(&self, _module: &dyn Module, _module_graph: &ModuleGraph) -> &[SourceType] {
    SOURCE_TYPES
  }

  fn size(&self, module: &dyn Module, _source_type: Option<&SourceType>) -> f64 {
    module.source().map_or(0, |source| source.size()) as f64
  }

  async fn generate(
    &self,
    source: &BoxSource,
    module: &dyn Module,
    generate_context: &mut GenerateContext,
  ) -> Result<BoxSource> {
    if matches!(
      generate_context.requested_source_type,
      SourceType::JavaScript
    ) {
      let mut source = ReplaceSource::new(source.clone());
      let compilation = generate_context.compilation;
      let mut init_fragments = vec![];
      let mut context = TemplateContext {
        compilation,
        module,
        init_fragments: &mut init_fragments,
        runtime: generate_context.runtime,
        concatenation_scope: generate_context.concatenation_scope.take(),
        data: generate_context.data,
        runtime_template: generate_context.runtime_template,
      };

      module.get_dependencies().iter().for_each(|dependency_id| {
        self.source_dependency(compilation, dependency_id, &mut source, &mut context)
      });

      if let Some(dependencies) = module.get_presentational_dependencies() {
        dependencies.iter().for_each(|dependency| {
          if let Some(template) = dependency
            .dependency_template()
            .and_then(|template_type| compilation.get_dependency_template(template_type))
          {
            template.render(dependency.as_ref(), &mut source, &mut context)
          } else {
            panic!(
              "Can not find dependency template of {:?}",
              dependency.dependency_template()
            );
          }
        });
      };

      module
        .get_blocks()
        .iter()
        .for_each(|block_id| self.source_block(compilation, block_id, &mut source, &mut context));
      generate_context.concatenation_scope = context.concatenation_scope.take();
      render_init_fragments(source.boxed(), init_fragments, generate_context)
    } else {
      panic!(
        "Unsupported source type: {:?}",
        generate_context.requested_source_type
      )
    }
  }

  fn get_concatenation_bailout_reason(
    &self,
    module: &dyn rspack_core::Module,
    _mg: &ModuleGraph,
    _cg: &ChunkGraph,
  ) -> Option<Cow<'static, str>> {
    // Only ES modules are valid for optimization
    if module.build_meta().exports_type != BuildMetaExportsType::Namespace {
      return Some("Module is not an ECMAScript module".into());
    }

    if let Some(deps) = module.get_presentational_dependencies() {
      if !deps.iter().any(|dep| {
        // https://github.com/webpack/webpack/blob/b9fb99c63ca433b24233e0bbc9ce336b47872c08/lib/javascript/JavascriptGenerator.js#L65-L74
        dep
          .as_any()
          .downcast_ref::<ESMCompatibilityDependency>()
          .is_some()
      }) {
        return Some("Module is not an ECMAScript module".into());
      }
    } else {
      return Some("Module is not an ECMAScript module".into());
    }

    if let Some(bailout) = module.build_info().module_concatenation_bailout.as_deref() {
      return Some(format!("Module uses {bailout}").into());
    }
    None
  }
}
