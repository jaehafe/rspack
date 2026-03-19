use std::sync::Arc;

use rspack_core::{ConstDependency, RuntimeGlobals, RuntimeRequirementsDependency};
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{CallExpr, Expr, MemberExpr, UnaryExpr},
};

use crate::{
  JavascriptParserCall, JavascriptParserCanRename, JavascriptParserEvaluateIdentifier,
  JavascriptParserEvaluateTypeof, JavascriptParserIdentifier, JavascriptParserMember,
  JavascriptParserPlugin, JavascriptParserPluginContext, JavascriptParserRename,
  JavascriptParserTypeof,
  utils::eval::{BasicEvaluatedExpression, evaluate_to_identifier, evaluate_to_string},
  visitors::JavascriptParserState,
};

pub struct AMDParserPlugin;

const DEFINE: &str = "define";
const REQUIRE: &str = "require";
const DEFINE_AMD: &str = "define.amd";
const REQUIRE_AMD: &str = "require.amd";

impl AMDParserPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParserState,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.config" || for_name == "requirejs.config" {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        call_expr.span.into(),
        "undefined".into(),
      )));
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParserState,
    expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.version" {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "\"0.0.0\"".into(),
      )));
      return Some(true);
    }
    if for_name == "requirejs.onError" {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        expr.span.into(),
        RuntimeGlobals::UNCAUGHT_ERROR_HANDLER,
      )));
      return Some(true);
    }

    // AMD
    if for_name == "define.amd" || for_name == "require.amd" {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        expr.span.into(),
        RuntimeGlobals::AMD_OPTIONS,
      )));
      return Some(true);
    }
    None
  }

  // The following is the logic from AMDPlugin, which mainly applies
  // AMDDefineDependencyParserPlugin and AMDRequireDependenciesBlockParserPlugin.
  // It also has some require.js related logic. I moved the logic here
  // to avoid creating a `AMDPlugin` with just a few lines of code.

  fn r#typeof(
    &self,
    parser: &mut JavascriptParserState,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == DEFINE || for_name == REQUIRE {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "\"function\"".into(),
      )));
      return Some(true);
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        expr.span.into(),
        "\"object\"".into(),
      )));
      return Some(true);
    }

    None
  }

  fn evaluate_typeof<'a>(
    &self,
    _parser: &mut JavascriptParserState,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if for_name == DEFINE || for_name == REQUIRE {
      return Some(evaluate_to_string(
        "function".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
    }

    if for_name == DEFINE_AMD || for_name == REQUIRE_AMD {
      return Some(evaluate_to_string(
        "object".to_string(),
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
    }

    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParserState,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == DEFINE {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        ident.span().into(),
        RuntimeGlobals::AMD_DEFINE,
      )));
      return Some(true);
    }
    None
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParserState,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression<'static>> {
    if for_name == DEFINE_AMD {
      return Some(evaluate_to_identifier(
        for_name.into(),
        "define".into(),
        Some(true),
        start,
        end,
      ));
    }

    if for_name == REQUIRE_AMD {
      return Some(evaluate_to_identifier(
        for_name.into(),
        "require".into(),
        Some(true),
        start,
        end,
      ));
    }

    None
  }

  fn can_rename(&self, _parser: &mut JavascriptParserState, for_name: &str) -> Option<bool> {
    if for_name == DEFINE {
      return Some(true);
    }
    None
  }

  fn rename(
    &self,
    parser: &mut JavascriptParserState,
    expr: &Expr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == DEFINE {
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        expr.span().into(),
        RuntimeGlobals::AMD_DEFINE,
      )));
      return Some(false);
    }
    None
  }
}

crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserCall,
  call(parser: &mut JavascriptParserState, call_expr: &CallExpr, for_name: &str) -> bool
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserMember,
  member(parser: &mut JavascriptParserState, expr: &MemberExpr, for_name: &str) -> bool
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserTypeof,
  r#typeof(parser: &mut JavascriptParserState, expr: &UnaryExpr, for_name: &str) -> bool
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserEvaluateTypeof,
  <'a>,
  evaluate_typeof(
    parser: &mut JavascriptParserState,
    expr: &'a UnaryExpr,
    for_name: &str
  ) -> BasicEvaluatedExpression<'a>
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserIdentifier,
  identifier(
    parser: &mut JavascriptParserState,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserEvaluateIdentifier,
  evaluate_identifier(
    parser: &mut JavascriptParserState,
    for_name: &str,
    start: u32,
    end: u32
  ) -> BasicEvaluatedExpression<'static>
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserCanRename,
  can_rename(parser: &mut JavascriptParserState, for_name: &str) -> bool
);
crate::impl_javascript_parser_hook!(
  AMDParserPlugin,
  JavascriptParserRename,
  rename(parser: &mut JavascriptParserState, expr: &Expr, for_name: &str) -> bool
);

impl JavascriptParserPlugin for AMDParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    for key in ["require.config", "requirejs.config"] {
      context.hooks.call.for_static(key).tap(self.clone());
    }
    for key in [
      "require.version",
      "requirejs.onError",
      DEFINE_AMD,
      REQUIRE_AMD,
    ] {
      context.hooks.member.for_static(key).tap(self.clone());
    }
    for key in [DEFINE, REQUIRE, DEFINE_AMD, REQUIRE_AMD] {
      context.hooks.r#typeof.for_static(key).tap(self.clone());
      context
        .hooks
        .evaluate_typeof
        .for_static(key)
        .tap(self.clone());
    }
    context
      .hooks
      .identifier
      .for_static(DEFINE)
      .tap(self.clone());
    for key in [DEFINE_AMD, REQUIRE_AMD] {
      context
        .hooks
        .evaluate_identifier
        .for_static(key)
        .tap(self.clone());
    }
    context
      .hooks
      .can_rename
      .for_static(DEFINE)
      .tap(self.clone());
    context.hooks.rename.for_static(DEFINE).tap(self);
  }
}
