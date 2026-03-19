use std::sync::Arc;

use rspack_core::ConstDependency;
use swc_core::atoms::wtf8::Wtf8;

use super::{JavascriptParserPlugin, JavascriptParserPluginContext, JavascriptParserProgram};
use crate::visitors::JavascriptParserState;

pub struct UseStrictPlugin;

impl UseStrictPlugin {
  fn program(
    &self,
    parser: &mut JavascriptParserState,
    ast: &swc_core::ecma::ast::Program,
  ) -> Option<bool> {
    let first = match ast {
      swc_core::ecma::ast::Program::Module(ast) => ast.body.first().and_then(|i| i.as_stmt()),
      swc_core::ecma::ast::Program::Script(ast) => ast.body.first(),
    }
    .and_then(|i| i.as_expr());
    if let Some(first) = first
      && first.expr.as_lit().and_then(|i| match i {
        swc_core::ecma::ast::Lit::Str(s) => Some(s.value.as_wtf8()),
        _ => None,
      }) == Some(Wtf8::from_str("use strict"))
    {
      // Remove "use strict" expression. It will be added later by the renderer again.
      // This is necessary in order to not break the strict mode when webpack prepends code.
      let dep = ConstDependency::new(first.span.into(), "".into());
      parser.add_presentational_dependency(Box::new(dep));
      parser.build_info.strict = true;
    }
    None
  }
}

crate::impl_javascript_parser_hook!(
  UseStrictPlugin,
  JavascriptParserProgram,
  program(parser: &mut JavascriptParserState, ast: &swc_core::ecma::ast::Program) -> bool
);

impl JavascriptParserPlugin for UseStrictPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.program.tap(self);
  }
}
