use std::sync::Arc;

use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use swc_core::ecma::ast::MemberExpr;

use super::{
  JavascriptParserEvaluateIdentifier, JavascriptParserMember, JavascriptParserPlugin,
  JavascriptParserPluginContext, JavascriptParserTypeof,
};
use crate::{
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier},
  visitors::{JavascriptParserState, expr_name},
};

pub struct CommonJsPlugin;

impl CommonJsPlugin {
  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParserState,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name == expr_name::MODULE_HOT {
      Some(evaluate_to_identifier(
        expr_name::MODULE_HOT.into(),
        expr_name::MODULE.into(),
        None,
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::MODULE {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "'object'".into(),
      )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParserState,
    _expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "module.id" {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
        RuntimeGlobals::MODULE_ID,
      )));
      parser.build_info.module_concatenation_bailout = Some(for_name.to_string());
      return Some(true);
    }

    if for_name == "module.loaded" {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::add_only(
        RuntimeGlobals::MODULE_LOADED,
      )));
      parser.build_info.module_concatenation_bailout = Some(for_name.to_string());
      return Some(true);
    }

    None
  }
}

crate::impl_javascript_parser_hook!(
  CommonJsPlugin,
  JavascriptParserEvaluateIdentifier,
  evaluate_identifier(
    parser: &mut JavascriptParserState,
    for_name: &str,
    start: u32,
    end: u32
  ) -> BasicEvaluatedExpression<'static>
);
crate::impl_javascript_parser_hook!(
  CommonJsPlugin,
  JavascriptParserTypeof,
  r#typeof(
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  CommonJsPlugin,
  JavascriptParserMember,
  member(parser: &mut JavascriptParserState, expr: &MemberExpr, for_name: &str) -> bool
);

impl JavascriptParserPlugin for CommonJsPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context
      .hooks
      .evaluate_identifier
      .for_static(expr_name::MODULE_HOT)
      .tap(self.clone());
    context
      .hooks
      .r#typeof
      .for_static(expr_name::MODULE)
      .tap(self.clone());
    context
      .hooks
      .member
      .for_static("module.id")
      .tap(self.clone());
    context.hooks.member.for_static("module.loaded").tap(self);
  }
}
