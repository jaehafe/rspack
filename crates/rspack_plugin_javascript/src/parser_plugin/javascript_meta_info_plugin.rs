use std::sync::Arc;

use rspack_util::atom::Atom;
use rustc_hash::FxHashSet;

use super::{
  JavascriptParserCall, JavascriptParserFinish, JavascriptParserPlugin,
  JavascriptParserPluginContext, TopLevelSymbol,
};
use crate::visitors::JavascriptParserState;

pub struct JavascriptMetaInfoPlugin;

impl JavascriptMetaInfoPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParserState,
    _expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "eval" {
      parser.build_info.module_concatenation_bailout = Some("eval()".into());
      if let Some(top_level_symbol) = parser.inner_graph.get_top_level_symbol() {
        parser.inner_graph.add_usage(
          TopLevelSymbol::global(),
          super::InnerGraphMapUsage::TopLevel(top_level_symbol),
        );
      } else {
        parser.inner_graph.bailout();
      }
    }

    None
  }

  fn finish(&self, parser: &mut JavascriptParserState) -> Option<bool> {
    if parser.build_info.top_level_declarations.is_none() {
      parser.build_info.top_level_declarations = Some(FxHashSet::default());
    }
    let variables: Vec<_> = parser
      .get_all_variables_from_current_scope()
      .map(|(name, _)| Atom::new(name))
      .collect();
    for name in variables {
      if parser.is_variable_defined(&name) {
        parser
          .build_info
          .top_level_declarations
          .as_mut()
          .expect("must have value")
          .insert(name);
      }
    }
    None
  }
}

crate::impl_javascript_parser_hook!(
  JavascriptMetaInfoPlugin,
  JavascriptParserCall,
  call(
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  JavascriptMetaInfoPlugin,
  JavascriptParserFinish,
  finish(parser: &mut JavascriptParserState) -> bool
);

impl JavascriptParserPlugin for JavascriptMetaInfoPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.call.for_static("eval").tap(self.clone());
    context.hooks.finish.tap(self);
  }
}
