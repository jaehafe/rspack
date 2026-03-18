use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, UnaryExpr},
};

use super::{
  JavascriptParserCall, JavascriptParserPlugin, JavascriptParserPluginContext,
  JavascriptParserTypeof,
};
use crate::{dependency::IsIncludeDependency, visitors::JavascriptParser};

const IS_INCLUDED: &str = "__webpack_is_included__";

pub struct IsIncludedPlugin;

impl IsIncludedPlugin {
  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    if name != IS_INCLUDED || expr.args.len() != 1 || expr.args[0].spread.is_some() {
      return None;
    }

    let request = parser.evaluate_expression(&expr.args[0].expr);
    if !request.is_string() {
      return None;
    }

    parser.add_dependency(Box::new(IsIncludeDependency::new(
      (expr.span().real_lo(), expr.span().real_hi()).into(),
      request.string().clone(),
    )));

    Some(true)
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'_>,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == IS_INCLUDED).then(|| {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        (expr.span().real_lo(), expr.span().real_hi()).into(),
        "'function'".into(),
      )));
      true
    })
  }
}

crate::impl_javascript_parser_hook!(
  IsIncludedPlugin,
  JavascriptParserCall,
  call(parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> bool
);
crate::impl_javascript_parser_hook!(
  IsIncludedPlugin,
  JavascriptParserTypeof,
  r#typeof(parser: &mut JavascriptParser<'_>, expr: &UnaryExpr, for_name: &str) -> bool
);

impl JavascriptParserPlugin for IsIncludedPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.call.for_static(IS_INCLUDED).tap(self.clone());
    context.hooks.r#typeof.for_static(IS_INCLUDED).tap(self);
  }
}
