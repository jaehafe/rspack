use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_plugin_javascript::{
  JavascriptParserPlugin, JavascriptParserPluginContext, JavascriptParserProgram,
  visitors::JavascriptParserState,
};
use swc_core::ecma::ast::{Expr, Lit, ModuleItem, Program, Stmt};

pub struct ReactDirectivesParserPlugin;

impl ReactDirectivesParserPlugin {
  fn process_statements<'a, I>(stmts: I, directives: &mut Vec<(String, swc_core::common::Span)>)
  where
    I: Iterator<Item = &'a Stmt>,
  {
    for stmt in stmts {
      let Stmt::Expr(expr_stmt) = stmt else { break };
      let Expr::Lit(Lit::Str(str_lit)) = &*expr_stmt.expr else {
        break;
      };

      let value = str_lit.value.to_string_lossy().to_string();
      if !value.starts_with("use ") {
        break;
      }

      let directive = format!("\"{value}\"");
      directives.push((directive, expr_stmt.span));
    }
  }
}

impl ReactDirectivesParserPlugin {
  fn program(&self, parser: &mut JavascriptParserState, ast: &Program) -> Option<bool> {
    let mut directives = Vec::new();

    match ast {
      Program::Module(module) => {
        let stmts = module.body.iter().filter_map(|item| {
          if let ModuleItem::Stmt(stmt) = item {
            Some(stmt)
          } else {
            None
          }
        });
        Self::process_statements(stmts, &mut directives);
      }
      Program::Script(script) => {
        Self::process_statements(script.body.iter(), &mut directives);
      }
    }

    if directives.is_empty() {
      return None;
    }

    parser.build_info.extras.insert(
      "react_directives".to_string(),
      serde_json::json!(directives.iter().map(|(d, _)| d).collect::<Vec<_>>()),
    );

    for (_, span) in directives {
      parser.add_presentational_dependency(Box::new(ConstDependency::new(span.into(), "".into())));
    }

    None
  }
}

struct ReactDirectivesParserPluginProgramTap(Arc<ReactDirectivesParserPlugin>);

impl JavascriptParserProgram for ReactDirectivesParserPluginProgramTap {
  fn run(
    &self,
    parser: &mut JavascriptParserState,
    ast: &Program,
  ) -> rspack_error::Result<Option<bool>> {
    Ok(self.0.program(parser, ast))
  }
}

impl JavascriptParserPlugin for ReactDirectivesParserPlugin {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>) {
    context
      .hooks
      .program
      .tap(ReactDirectivesParserPluginProgramTap(self));
  }
}
