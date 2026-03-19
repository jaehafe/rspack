use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{Ident, MemberExpr},
};

use super::{
  JavascriptParserIdentifier, JavascriptParserMemberChain, JavascriptParserPlugin,
  JavascriptParserPluginContext,
};
use crate::{dependency::ExportInfoDependency, visitors::JavascriptParserState};

const EXPORTS_INFO: &str = "__webpack_exports_info__";

pub struct ExportsInfoApiPlugin;

impl ExportsInfoApiPlugin {
  fn member_chain(
    &self,
    parser: &mut JavascriptParserState,
    member_expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    let len = members.len();
    if len >= 1 && for_name == EXPORTS_INFO {
      let prop = members[len - 1].clone();
      let dep = Box::new(ExportInfoDependency::new(
        member_expr.span.real_lo(),
        member_expr.span.real_hi(),
        members.iter().take(len - 1).cloned().collect::<Vec<_>>(),
        prop,
      ));
      parser.add_presentational_dependency(dep);
      Some(true)
    } else {
      None
    }
  }

  fn identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParserState,
    expr: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == EXPORTS_INFO {
      let dep = Box::new(ConstDependency::new(expr.span.into(), "true".into()));
      parser.add_presentational_dependency(dep);
      Some(true)
    } else {
      None
    }
  }
}

crate::impl_javascript_parser_hook!(
  ExportsInfoApiPlugin,
  JavascriptParserMemberChain,
  member_chain(
    parser: &mut JavascriptParserState,
    member_expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span]
  ) -> bool
);
crate::impl_javascript_parser_hook!(
  ExportsInfoApiPlugin,
  JavascriptParserIdentifier,
  identifier(
    parser: &mut crate::visitors::JavascriptParserState,
    expr: &Ident,
    for_name: &str
  ) -> bool
);

impl JavascriptParserPlugin for ExportsInfoApiPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context
      .hooks
      .member_chain
      .for_static(EXPORTS_INFO)
      .tap(self.clone());
    context.hooks.identifier.for_static(EXPORTS_INFO).tap(self);
  }
}
