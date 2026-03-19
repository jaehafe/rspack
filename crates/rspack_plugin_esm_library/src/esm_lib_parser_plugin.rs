use std::sync::Arc;

use rspack_core::DependencyType;
use rspack_plugin_javascript::{
  JavascriptParserFinish, JavascriptParserPlugin, JavascriptParserPluginContext,
  dependency::ESMCompatibilityDependency, visitors::JavascriptParserState,
};
pub struct EsmLibParserPlugin;

impl EsmLibParserPlugin {
  fn finish(&self, parser: &mut JavascriptParserState) -> Option<bool> {
    if parser.module_type.is_js_auto()
      && matches!(
        parser.build_meta.exports_type,
        rspack_core::BuildMetaExportsType::Unset
      )
      && !parser.get_dependencies().iter().any(|dep| {
        matches!(
          dep.dependency_type(),
          DependencyType::CjsExportRequire
            | DependencyType::CjsExports
            | DependencyType::CjsFullRequire
            | DependencyType::CjsRequire
            | DependencyType::CjsSelfReference
            | DependencyType::CommonJSRequireContext
            | DependencyType::ModuleDecorator
        )
      })
    {
      // make module without any exports or module accessing not bail out
      parser.build_meta.exports_type = rspack_core::BuildMetaExportsType::Namespace;
      parser.add_presentational_dependency(Box::new(ESMCompatibilityDependency));
    }

    None
  }
}

struct EsmLibParserPluginFinishTap(Arc<EsmLibParserPlugin>);

impl JavascriptParserFinish for EsmLibParserPluginFinishTap {
  fn run(&self, parser: &mut JavascriptParserState) -> rspack_error::Result<Option<bool>> {
    Ok(self.0.finish(parser))
  }
}

impl JavascriptParserPlugin for EsmLibParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.finish.tap(EsmLibParserPluginFinishTap(self));
  }
}
