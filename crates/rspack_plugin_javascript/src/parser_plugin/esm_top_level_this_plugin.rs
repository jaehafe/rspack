use std::sync::Arc;

use rspack_core::ConstDependency;

use super::{JavascriptParserPlugin, JavascriptParserPluginContext, JavascriptParserThis};
use crate::visitors::JavascriptParserState;

pub struct ESMTopLevelThisParserPlugin;

impl ESMTopLevelThisParserPlugin {
  fn this(
    &self,
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::ThisExpr,
    _for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && parser.is_top_level_this()).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "undefined".into(),
      )));
      true
    })
  }
}

crate::impl_javascript_parser_hook!(
  ESMTopLevelThisParserPlugin,
  JavascriptParserThis,
  this(
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::ThisExpr,
    for_name: &str
  ) -> bool
);

impl JavascriptParserPlugin for ESMTopLevelThisParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.this.for_static("this").tap(self);
  }
}
