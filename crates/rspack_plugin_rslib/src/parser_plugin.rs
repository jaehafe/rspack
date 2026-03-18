use std::sync::Arc;

use rspack_plugin_javascript::{
  JavascriptParserMember, JavascriptParserPlugin, JavascriptParserPluginContext,
  JavascriptParserTypeof, visitors::JavascriptParser,
};
use swc_core::ecma::ast::MemberExpr;

#[derive(PartialEq, Debug, Default)]
pub struct RslibParserPlugin {
  intercept_api_plugin: bool,
}

impl RslibParserPlugin {
  pub fn new(intercept_api_plugin: bool) -> Self {
    Self {
      intercept_api_plugin,
    }
  }
}

impl RslibParserPlugin {
  fn member(
    &self,
    _parser: &mut JavascriptParser,
    _member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.cache"
      || for_name == "require.extensions"
      || for_name == "require.config"
      || for_name == "require.version"
      || for_name == "require.include"
      || for_name == "require.onError"
    {
      return Some(true);
    }
    None
  }

  fn r#typeof(
    &self,
    _parser: &mut JavascriptParser,
    _expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "module" {
      Some(false)
    } else {
      None
    }
  }
}

#[derive(Clone)]
struct RslibParserPluginTap(Arc<RslibParserPlugin>);

impl JavascriptParserMember for RslibParserPluginTap {
  fn run(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    for_name: &str,
  ) -> rspack_error::Result<Option<bool>> {
    Ok(self.0.member(parser, member_expr, for_name))
  }
}

impl JavascriptParserTypeof for RslibParserPluginTap {
  fn run(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> rspack_error::Result<Option<bool>> {
    Ok(self.0.r#typeof(parser, expr, for_name))
  }
}

impl JavascriptParserPlugin for RslibParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    let tap = RslibParserPluginTap(self);
    for key in [
      "require.cache",
      "require.extensions",
      "require.config",
      "require.version",
      "require.include",
      "require.onError",
    ] {
      context.hooks.member.for_static(key).tap(tap.clone());
    }
    context.hooks.r#typeof.for_static("module").tap(tap);
  }
}
