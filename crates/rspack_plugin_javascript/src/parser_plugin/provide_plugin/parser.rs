use std::sync::Arc;

use cow_utils::CowUtils;
use itertools::Itertools;
use rspack_core::DependencyRange;
use rustc_hash::FxHashSet as HashSet;
use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
};

use super::{
  super::{
    JavascriptParserCall, JavascriptParserCanRename, JavascriptParserIdentifier,
    JavascriptParserMember, JavascriptParserPlugin, JavascriptParserPluginContext,
    JavascriptParserRename,
  },
  ProvideValue, VALUE_DEP_PREFIX,
};
use crate::{dependency::ProvideDependency, visitors::JavascriptParserState};

const SOURCE_DOT: &str = r#"."#;
const MODULE_DOT: &str = r#"_dot_"#;

pub struct ProvideParserPlugin {
  provide: Arc<ProvideValue>,
  names: Arc<HashSet<String>>,
}

impl ProvideParserPlugin {
  pub fn new(provide: Arc<ProvideValue>, names: Arc<HashSet<String>>) -> Self {
    Self { provide, names }
  }

  fn add_provide_dep(&self, name: &str, span: Span, parser: &mut JavascriptParserState) -> bool {
    if let Some(requests) = self.provide.get(name) {
      let name_identifier = if name.contains(SOURCE_DOT) {
        format!(
          "__rspack_provide_{}",
          name.cow_replace(SOURCE_DOT, MODULE_DOT)
        )
      } else {
        name.to_string()
      };
      let range = DependencyRange::from(span);
      let loc = parser.to_dependency_location(range);
      let dep = ProvideDependency::new(
        range,
        Atom::from(requests[0].as_str()),
        name_identifier,
        requests[1..]
          .iter()
          .map(|s| Atom::from(s.as_str()))
          .collect_vec(),
        loc,
      );
      parser.add_dependency(Box::new(dep));

      // add value dependency
      let cache_key = format!("{VALUE_DEP_PREFIX}{name}");
      parser
        .build_info
        .value_dependencies
        .insert(cache_key, requests.join("."));
      return true;
    }
    false
  }
}

impl ProvideParserPlugin {
  fn can_rename(&self, _parser: &mut JavascriptParserState, str: &str) -> Option<bool> {
    self.names.contains(str).then_some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self.add_provide_dep(for_name, expr.callee.span(), parser) {
      // FIXME: webpack use `walk_expression` here
      parser.walk_expr_or_spread(&expr.args);
      return Some(true);
    }
    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, expr.span(), parser)
      .then_some(true)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParserState,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, ident.span, parser)
      .then_some(true)
  }

  fn rename(
    &self,
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::Expr,
    for_name: &str,
  ) -> Option<bool> {
    self
      .add_provide_dep(for_name, expr.span(), parser)
      .then_some(false)
  }
}

crate::impl_javascript_parser_hook!(
  ProvideParserPlugin,
  JavascriptParserCanRename,
  can_rename(parser: &mut JavascriptParserState, str: &str) -> bool
);
crate::impl_javascript_parser_hook!(
  ProvideParserPlugin,
  JavascriptParserCall,
  call(
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::CallExpr,
    for_name: &str
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  ProvideParserPlugin,
  JavascriptParserMember,
  member(
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::MemberExpr,
    for_name: &str
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  ProvideParserPlugin,
  JavascriptParserIdentifier,
  identifier(
    parser: &mut JavascriptParserState,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  ProvideParserPlugin,
  JavascriptParserRename,
  rename(
    parser: &mut JavascriptParserState,
    expr: &swc_core::ecma::ast::Expr,
    for_name: &str
  ) -> bool
);

impl JavascriptParserPlugin for ProvideParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context.hooks.can_rename.reserve(self.names.len());
    for key in self.names.iter() {
      context
        .hooks
        .can_rename
        .r#for(key.as_str())
        .tap(self.clone());
    }

    let provide_count = self.provide.len();
    context.hooks.rename.reserve(provide_count);
    context.hooks.call.reserve(provide_count);
    context.hooks.member.reserve(provide_count);
    context.hooks.identifier.reserve(provide_count);
    for key in self.provide.keys() {
      let key = key.as_str();
      context.hooks.rename.r#for(key).tap(self.clone());
      context.hooks.call.r#for(key).tap(self.clone());
      context.hooks.member.r#for(key).tap(self.clone());
      context.hooks.identifier.r#for(key).tap(self.clone());
    }
  }
}
