use rspack_util::SpanExt;
use swc_core::ecma::ast::ArrayLit;

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParserState;

#[inline]
pub fn eval_array_expression<'a>(
  scanner: &mut JavascriptParserState,
  expr: &'a ArrayLit,
) -> Option<BasicEvaluatedExpression<'a>> {
  let mut items = vec![];

  for elem in &expr.elems {
    if let Some(elem) = elem
      && elem.spread.is_none()
    {
      items.push(scanner.evaluate_expression(&elem.expr));
    } else {
      return None;
    }
  }

  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
  res.set_items(items);
  Some(res)
}
