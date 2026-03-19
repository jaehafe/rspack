use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_plugin_javascript::{
  JavascriptParserPlugin, JavascriptParserPluginContext, JavascriptParserProgram,
  visitors::JavascriptParserState,
};
use swc_core::ecma::ast::Program;

pub struct HashbangParserPlugin;

impl HashbangParserPlugin {
  fn program(&self, parser: &mut JavascriptParserState, ast: &Program) -> Option<bool> {
    let hashbang = ast
      .as_module()
      .and_then(|m| m.shebang.as_ref())
      .or_else(|| ast.as_script().and_then(|s| s.shebang.as_ref()))?;

    // Normalize hashbang to always include "#!" prefix
    // SWC may omit the leading "#!" in the shebang value
    let normalized_hashbang = if hashbang.starts_with("#!") {
      hashbang.to_string()
    } else {
      format!("#!{hashbang}")
    };

    // Store hashbang in build_info for later use during rendering
    parser.build_info.extras.insert(
      "hashbang".to_string(),
      serde_json::Value::String(normalized_hashbang),
    );

    // Remove hashbang from source code
    // If SWC omitted "#!", we still need to remove those two characters
    let replace_len = if hashbang.starts_with("#!") {
      hashbang.len() as u32
    } else {
      hashbang.len() as u32 + 2 // include "#!"
    };

    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (0, replace_len).into(),
      "".into(),
    )));

    None
  }
}

struct HashbangParserPluginProgramTap(Arc<HashbangParserPlugin>);

impl JavascriptParserProgram for HashbangParserPluginProgramTap {
  fn run(
    &self,
    parser: &mut JavascriptParserState,
    ast: &Program,
  ) -> rspack_error::Result<Option<bool>> {
    Ok(self.0.program(parser, ast))
  }
}

impl JavascriptParserPlugin for HashbangParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context
      .hooks
      .program
      .tap(HashbangParserPluginProgramTap(self));
  }
}
