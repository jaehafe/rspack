use rspack_core::{
  CompilerOptions, JavascriptParserOptions, ModuleLayer, ModuleType, ResourceData,
};
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{
    AssignExpr, BinExpr, CallExpr, Callee, ClassMember, CondExpr, Expr, IfStmt, MemberExpr,
    OptChainExpr, UnaryExpr, VarDeclarator,
  },
};

use super::hook::{HookMap, define_parser_sync_bail_hook, define_parser_sync_hook};
use crate::{
  parser_and_generator::ParserRuntimeRequirementsData,
  parser_plugin::{BoxJavascriptParserPlugin, JavascriptParserPluginContext},
  utils::eval::BasicEvaluatedExpression,
  visitors::{
    ClassDeclOrExpr, DestructuringAssignmentProperty, ExportDefaultDeclaration,
    ExportDefaultExpression, ExportImport, ExportLocal, ExportedVariableInfo, JavascriptParser,
    Statement, VariableDeclaration,
  },
};

define_parser_sync_bail_hook!(
  JavascriptParserPreStatement,
  JavascriptParserPreStatementHook,
  (parser: &mut JavascriptParser, stmt: Statement) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserBlockPreStatement,
  JavascriptParserBlockPreStatementHook,
  (parser: &mut JavascriptParser, stmt: Statement) -> bool
);
define_parser_sync_hook!(
  JavascriptParserTopLevelAwaitExpr,
  JavascriptParserTopLevelAwaitExprHook,
  (parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::AwaitExpr)
);
define_parser_sync_hook!(
  JavascriptParserTopLevelForOfAwaitStmt,
  JavascriptParserTopLevelForOfAwaitStmtHook,
  (parser: &mut JavascriptParser, stmt: &swc_core::ecma::ast::ForOfStmt)
);
define_parser_sync_bail_hook!(
  JavascriptParserCanRename,
  JavascriptParserCanRenameHook,
  (parser: &mut JavascriptParser, name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserRename,
  JavascriptParserRenameHook,
  (parser: &mut JavascriptParser, expr: &Expr, name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserProgram,
  JavascriptParserProgramHook,
  (parser: &mut JavascriptParser, ast: &swc_core::ecma::ast::Program) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserStatement,
  JavascriptParserStatementHook,
  (parser: &mut JavascriptParser, stmt: Statement) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserUnusedStatement,
  JavascriptParserUnusedStatementHook,
  (parser: &mut JavascriptParser, stmt: Statement) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserModuleDeclaration,
  JavascriptParserModuleDeclarationHook,
  (parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserBlockPreModuleDeclaration,
  JavascriptParserBlockPreModuleDeclarationHook,
  (parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserPreDeclarator,
  JavascriptParserPreDeclaratorHook,
  (parser: &mut JavascriptParser, declarator: &VarDeclarator, declaration: VariableDeclaration<'_>) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserEvaluateIdentifier,
  JavascriptParserEvaluateIdentifierHook,
  (parser: &mut JavascriptParser, for_name: &str, start: u32, end: u32) -> BasicEvaluatedExpression<'static>
);
define_parser_sync_bail_hook!(
  JavascriptParserCanCollectDestructuringAssignmentProperties,
  JavascriptParserCanCollectDestructuringAssignmentPropertiesHook,
  (parser: &mut JavascriptParser, expr: &Expr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserPattern,
  JavascriptParserPatternHook,
  (parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserCall,
  JavascriptParserCallHook,
  (parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserCallMemberChain,
  JavascriptParserCallMemberChainHook,
  (parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserMember,
  JavascriptParserMemberHook,
  (parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserMemberChain,
  JavascriptParserMemberChainHook,
  (parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserUnhandledExpressionMemberChain,
  JavascriptParserUnhandledExpressionMemberChainHook,
  (parser: &mut JavascriptParser, root_info: &ExportedVariableInfo, expr: &MemberExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserMemberChainOfCallMemberChain,
  JavascriptParserMemberChainOfCallMemberChainHook,
  (parser: &mut JavascriptParser, member_expr: &MemberExpr, callee_members: &[Atom], call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserCallMemberChainOfCallMemberChain,
  JavascriptParserCallMemberChainOfCallMemberChainHook,
  (parser: &mut JavascriptParser, call_expr: &CallExpr, callee_members: &[Atom], inner_call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserTypeof,
  JavascriptParserTypeofHook,
  (parser: &mut JavascriptParser, expr: &UnaryExpr, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExpressionLogicalOperator,
  JavascriptParserExpressionLogicalOperatorHook,
  (parser: &mut JavascriptParser, expr: &BinExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserBinaryExpression,
  JavascriptParserBinaryExpressionHook,
  (parser: &mut JavascriptParser, expr: &BinExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserStatementIf,
  JavascriptParserStatementIfHook,
  (parser: &mut JavascriptParser, expr: &IfStmt) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserClassExtendsExpression,
  JavascriptParserClassExtendsExpressionHook,
  (parser: &mut JavascriptParser, super_class: &Expr, class_decl_or_expr: ClassDeclOrExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserClassBodyElement,
  JavascriptParserClassBodyElementHook,
  (parser: &mut JavascriptParser, element: &ClassMember, class_decl_or_expr: ClassDeclOrExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserClassBodyValue,
  JavascriptParserClassBodyValueHook,
  (parser: &mut JavascriptParser, element: &ClassMember, expr_span: Span, class_decl_or_expr: ClassDeclOrExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserDeclarator,
  JavascriptParserDeclaratorHook,
  (parser: &mut JavascriptParser, expr: &VarDeclarator, stmt: VariableDeclaration<'_>) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserNewExpression,
  JavascriptParserNewExpressionHook,
  (parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::NewExpr, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserIdentifier,
  JavascriptParserIdentifierHook,
  (parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserThis,
  JavascriptParserThisHook,
  (parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::ThisExpr, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserAssign,
  JavascriptParserAssignHook,
  (parser: &mut JavascriptParser, expr: &AssignExpr, for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserAssignMemberChain,
  JavascriptParserAssignMemberChainHook,
  (parser: &mut JavascriptParser, expr: &AssignExpr, members: &[Atom], for_name: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserImportCall,
  JavascriptParserImportCallHook,
  (parser: &mut JavascriptParser, expr: &CallExpr, import_then: Option<&CallExpr>, members: Option<(&[Atom], bool)>) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserMetaProperty,
  JavascriptParserMetaPropertyHook,
  (parser: &mut JavascriptParser, root_name: &Atom, span: Span) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserImport,
  JavascriptParserImportHook,
  (parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &str) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserImportSpecifier,
  JavascriptParserImportSpecifierHook,
  (parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &swc_core::atoms::Atom, export_name: Option<&Atom>, identifier_name: &Atom) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExportImport,
  JavascriptParserExportImportHook,
  (parser: &mut JavascriptParser, statement: ExportImport, source: &Atom) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExport,
  JavascriptParserExportHook,
  (parser: &mut JavascriptParser, statement: ExportLocal) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExportImportSpecifier,
  JavascriptParserExportImportSpecifierHook,
  (parser: &mut JavascriptParser, statement: ExportImport, source: &Atom, local_id: Option<&Atom>, export_name: Option<&Atom>, export_name_span: Option<Span>) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExportSpecifier,
  JavascriptParserExportSpecifierHook,
  (parser: &mut JavascriptParser, statement: ExportLocal, local_id: &Atom, export_name: &Atom, export_name_span: Span) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExportExpression,
  JavascriptParserExportExpressionHook,
  (parser: &mut JavascriptParser, statement: ExportDefaultDeclaration, expr: ExportDefaultExpression) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserOptionalChaining,
  JavascriptParserOptionalChainingHook,
  (parser: &mut JavascriptParser, expr: &OptChainExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserExpressionConditionalOperation,
  JavascriptParserExpressionConditionalOperationHook,
  (parser: &mut JavascriptParser, expr: &CondExpr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserFinish,
  JavascriptParserFinishHook,
  (parser: &mut JavascriptParser) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserIsPure,
  JavascriptParserIsPureHook,
  (parser: &mut JavascriptParser, expr: &Expr) -> bool
);
define_parser_sync_bail_hook!(
  JavascriptParserImportMetaPropertyInDestructuring,
  JavascriptParserImportMetaPropertyInDestructuringHook,
  (parser: &mut JavascriptParser, property: &DestructuringAssignmentProperty) -> String
);
define_parser_sync_bail_hook!(
  JavascriptParserEvaluate,
  JavascriptParserEvaluateHook,
  <'a>,
  (parser: &mut JavascriptParser, expr: &'a Expr) -> BasicEvaluatedExpression<'a>
);
define_parser_sync_bail_hook!(
  JavascriptParserEvaluateTypeof,
  JavascriptParserEvaluateTypeofHook,
  <'a>,
  (parser: &mut JavascriptParser, expr: &'a UnaryExpr, for_name: &str) -> BasicEvaluatedExpression<'a>
);
define_parser_sync_bail_hook!(
  JavascriptParserEvaluateTypeofAny,
  JavascriptParserEvaluateTypeofAnyHook,
  <'a>,
  (parser: &mut JavascriptParser, expr: &'a UnaryExpr, for_name: &str) -> BasicEvaluatedExpression<'a>
);
define_parser_sync_bail_hook!(
  JavascriptParserEvaluateCallExpression,
  JavascriptParserEvaluateCallExpressionHook,
  <'a>,
  (parser: &mut JavascriptParser, name: &str, expr: &'a CallExpr) -> BasicEvaluatedExpression<'a>
);

pub trait JavascriptParserEvaluateCallExpressionMember {
  fn run<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> rspack_error::Result<Option<BasicEvaluatedExpression<'a>>>;
}

#[derive(Default)]
pub struct JavascriptParserEvaluateCallExpressionMemberHook {
  taps: Vec<Box<dyn JavascriptParserEvaluateCallExpressionMember + Send + Sync>>,
}

impl std::fmt::Debug for JavascriptParserEvaluateCallExpressionMemberHook {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JavascriptParserEvaluateCallExpressionMemberHook")
  }
}

impl JavascriptParserEvaluateCallExpressionMemberHook {
  pub fn call<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> rspack_error::Result<Option<BasicEvaluatedExpression<'a>>> {
    for tap in &self.taps {
      if let Some(result) = tap.run(parser, property, expr, param.clone())? {
        return Ok(Some(result));
      }
    }

    Ok(None)
  }

  pub fn tap(
    &mut self,
    tap: impl JavascriptParserEvaluateCallExpressionMember + Send + Sync + 'static,
  ) {
    self.taps.push(Box::new(tap));
  }
}

macro_rules! call_sync {
  ($field:ident($($arg:ident: $ty:ty),* $(,)?)) => {
    pub fn $field(&self, $($arg: $ty),*) {
      self
        .$field
        .call($($arg),*)
        .expect(concat!(stringify!($field), " parser hook should not fail"));
    }
  };
}

macro_rules! call_sync_bail {
  ($field:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
    pub fn $field(&self, $($arg: $ty),*) -> Option<$ret> {
      self
        .$field
        .call($($arg),*)
        .expect(concat!(stringify!($field), " parser hook should not fail"))
    }
  };
}

macro_rules! call_sync_bail_map {
  ($field:ident($($arg:ident: $ty:ty),* $(,)?), $for_name:ident -> $ret:ty) => {
    #[allow(clippy::too_many_arguments)]
    pub fn $field(&self, $($arg: $ty),*) -> Option<$ret> {
      self
        .$field
        .get($for_name)
        .and_then(|hook| {
          hook
            .call($($arg),*)
            .expect(concat!(stringify!($field), " parser hook should not fail"))
        })
    }
  };
}

#[derive(Debug, Default)]
pub struct JavascriptParserHooks {
  pub pre_statement: JavascriptParserPreStatementHook,
  pub block_pre_statement: JavascriptParserBlockPreStatementHook,
  pub top_level_await_expr: JavascriptParserTopLevelAwaitExprHook,
  pub top_level_for_of_await_stmt: JavascriptParserTopLevelForOfAwaitStmtHook,
  pub can_rename: HookMap<JavascriptParserCanRenameHook>,
  pub rename: HookMap<JavascriptParserRenameHook>,
  pub program: JavascriptParserProgramHook,
  pub statement: JavascriptParserStatementHook,
  pub unused_statement: JavascriptParserUnusedStatementHook,
  pub module_declaration: JavascriptParserModuleDeclarationHook,
  pub block_pre_module_declaration: JavascriptParserBlockPreModuleDeclarationHook,
  pub pre_declarator: JavascriptParserPreDeclaratorHook,
  pub evaluate: JavascriptParserEvaluateHook,
  pub evaluate_typeof: HookMap<JavascriptParserEvaluateTypeofHook>,
  pub evaluate_typeof_any: JavascriptParserEvaluateTypeofAnyHook,
  pub evaluate_call_expression: JavascriptParserEvaluateCallExpressionHook,
  pub evaluate_call_expression_member: JavascriptParserEvaluateCallExpressionMemberHook,
  pub evaluate_identifier: HookMap<JavascriptParserEvaluateIdentifierHook>,
  pub can_collect_destructuring_assignment_properties:
    JavascriptParserCanCollectDestructuringAssignmentPropertiesHook,
  pub pattern: HookMap<JavascriptParserPatternHook>,
  pub call: HookMap<JavascriptParserCallHook>,
  pub call_member_chain: HookMap<JavascriptParserCallMemberChainHook>,
  pub member: HookMap<JavascriptParserMemberHook>,
  pub member_chain: HookMap<JavascriptParserMemberChainHook>,
  pub unhandled_expression_member_chain: JavascriptParserUnhandledExpressionMemberChainHook,
  pub member_chain_of_call_member_chain: HookMap<JavascriptParserMemberChainOfCallMemberChainHook>,
  pub call_member_chain_of_call_member_chain:
    HookMap<JavascriptParserCallMemberChainOfCallMemberChainHook>,
  pub r#typeof: HookMap<JavascriptParserTypeofHook>,
  pub expression_logical_operator: JavascriptParserExpressionLogicalOperatorHook,
  pub binary_expression: JavascriptParserBinaryExpressionHook,
  pub statement_if: JavascriptParserStatementIfHook,
  pub class_extends_expression: JavascriptParserClassExtendsExpressionHook,
  pub class_body_element: JavascriptParserClassBodyElementHook,
  pub class_body_value: JavascriptParserClassBodyValueHook,
  pub declarator: JavascriptParserDeclaratorHook,
  pub new_expression: HookMap<JavascriptParserNewExpressionHook>,
  pub identifier: HookMap<JavascriptParserIdentifierHook>,
  pub this: HookMap<JavascriptParserThisHook>,
  pub assign: HookMap<JavascriptParserAssignHook>,
  pub assign_member_chain: HookMap<JavascriptParserAssignMemberChainHook>,
  pub import_call: JavascriptParserImportCallHook,
  pub meta_property: JavascriptParserMetaPropertyHook,
  pub import: JavascriptParserImportHook,
  pub import_specifier: JavascriptParserImportSpecifierHook,
  pub export_import: JavascriptParserExportImportHook,
  pub export: JavascriptParserExportHook,
  pub export_import_specifier: JavascriptParserExportImportSpecifierHook,
  pub export_specifier: JavascriptParserExportSpecifierHook,
  pub export_expression: JavascriptParserExportExpressionHook,
  pub optional_chaining: JavascriptParserOptionalChainingHook,
  pub expression_conditional_operation: JavascriptParserExpressionConditionalOperationHook,
  pub finish: JavascriptParserFinishHook,
  pub is_pure: JavascriptParserIsPureHook,
  pub import_meta_property_in_destructuring: JavascriptParserImportMetaPropertyInDestructuringHook,
}

impl JavascriptParserHooks {
  pub fn new(
    plugins: Vec<BoxJavascriptParserPlugin>,
    compiler_options: &CompilerOptions,
    javascript_options: &JavascriptParserOptions,
    parser_runtime_requirements: &ParserRuntimeRequirementsData,
    module_type: &ModuleType,
    module_layer: Option<&ModuleLayer>,
    resource_data: &ResourceData,
  ) -> Self {
    let mut hooks = Self::default();
    let mut context = JavascriptParserPluginContext {
      hooks: &mut hooks,
      compiler_options,
      javascript_options,
      parser_runtime_requirements,
      module_type,
      module_layer,
      resource_data,
    };
    for plugin in plugins {
      plugin.apply(&mut context);
    }
    hooks
  }

  call_sync!(top_level_await_expr(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::AwaitExpr));
  call_sync!(top_level_for_of_await_stmt(parser: &mut JavascriptParser, stmt: &swc_core::ecma::ast::ForOfStmt));

  call_sync_bail!(program(parser: &mut JavascriptParser, ast: &swc_core::ecma::ast::Program) -> bool);
  call_sync_bail!(finish(parser: &mut JavascriptParser) -> bool);
  call_sync_bail!(block_pre_module_declaration(parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool);
  call_sync_bail!(module_declaration(parser: &mut JavascriptParser, decl: &swc_core::ecma::ast::ModuleDecl) -> bool);
  call_sync_bail!(can_collect_destructuring_assignment_properties(parser: &mut JavascriptParser, expr: &Expr) -> bool);
  call_sync_bail!(unhandled_expression_member_chain(parser: &mut JavascriptParser, root_info: &ExportedVariableInfo, expr: &MemberExpr) -> bool);
  call_sync_bail!(expression_logical_operator(parser: &mut JavascriptParser, expr: &BinExpr) -> bool);
  call_sync_bail!(binary_expression(parser: &mut JavascriptParser, expr: &BinExpr) -> bool);
  call_sync_bail!(statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(unused_statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(statement_if(parser: &mut JavascriptParser, expr: &IfStmt) -> bool);
  call_sync_bail!(declarator(parser: &mut JavascriptParser, expr: &VarDeclarator, stmt: VariableDeclaration<'_>) -> bool);
  call_sync_bail!(class_extends_expression(parser: &mut JavascriptParser, super_class: &Expr, class_decl_or_expr: ClassDeclOrExpr) -> bool);
  call_sync_bail!(class_body_element(parser: &mut JavascriptParser, element: &ClassMember, class_decl_or_expr: ClassDeclOrExpr) -> bool);
  call_sync_bail!(class_body_value(parser: &mut JavascriptParser, element: &ClassMember, expr_span: Span, class_decl_or_expr: ClassDeclOrExpr) -> bool);
  call_sync_bail!(pre_declarator(parser: &mut JavascriptParser, declarator: &VarDeclarator, declaration: VariableDeclaration<'_>) -> bool);
  call_sync_bail!(pre_statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(block_pre_statement(parser: &mut JavascriptParser, stmt: Statement) -> bool);
  call_sync_bail!(import_call(parser: &mut JavascriptParser, expr: &CallExpr, import_then: Option<&CallExpr>, members: Option<(&[Atom], bool)>) -> bool);
  call_sync_bail!(meta_property(parser: &mut JavascriptParser, root_name: &Atom, span: Span) -> bool);
  call_sync_bail!(import(parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &str) -> bool);
  call_sync_bail!(import_specifier(parser: &mut JavascriptParser, statement: &swc_core::ecma::ast::ImportDecl, source: &swc_core::atoms::Atom, export_name: Option<&Atom>, identifier_name: &Atom) -> bool);
  call_sync_bail!(export_import(parser: &mut JavascriptParser, statement: ExportImport, source: &Atom) -> bool);
  call_sync_bail!(export(parser: &mut JavascriptParser, statement: ExportLocal) -> bool);
  call_sync_bail!(export_import_specifier(parser: &mut JavascriptParser, statement: ExportImport, source: &Atom, local_id: Option<&Atom>, export_name: Option<&Atom>, export_name_span: Option<Span>) -> bool);
  call_sync_bail!(export_specifier(parser: &mut JavascriptParser, statement: ExportLocal, local_id: &Atom, export_name: &Atom, export_name_span: Span) -> bool);
  call_sync_bail!(export_expression(parser: &mut JavascriptParser, statement: ExportDefaultDeclaration, expr: ExportDefaultExpression) -> bool);
  call_sync_bail!(optional_chaining(parser: &mut JavascriptParser, expr: &OptChainExpr) -> bool);
  call_sync_bail!(expression_conditional_operation(parser: &mut JavascriptParser, expr: &CondExpr) -> bool);
  call_sync_bail!(is_pure(parser: &mut JavascriptParser, expr: &Expr) -> bool);
  call_sync_bail!(import_meta_property_in_destructuring(parser: &mut JavascriptParser, property: &DestructuringAssignmentProperty) -> String);

  call_sync_bail_map!(member(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(member_chain(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::MemberExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]), for_name -> bool);
  call_sync_bail_map!(call_member_chain(parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str, members: &[Atom], members_optionals: &[bool], member_ranges: &[Span]), for_name -> bool);
  call_sync_bail_map!(member_chain_of_call_member_chain(parser: &mut JavascriptParser, member_expr: &MemberExpr, callee_members: &[Atom], call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str), for_name -> bool);
  call_sync_bail_map!(call_member_chain_of_call_member_chain(parser: &mut JavascriptParser, call_expr: &CallExpr, callee_members: &[Atom], inner_call_expr: &CallExpr, members: &[Atom], member_ranges: &[Span], for_name: &str), for_name -> bool);
  call_sync_bail_map!(assign(parser: &mut JavascriptParser, expr: &AssignExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(assign_member_chain(parser: &mut JavascriptParser, expr: &AssignExpr, members: &[Atom], for_name: &str), for_name -> bool);
  call_sync_bail_map!(r#typeof(parser: &mut JavascriptParser, expr: &UnaryExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(new_expression(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::NewExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(identifier(parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str), for_name -> bool);
  call_sync_bail_map!(this(parser: &mut JavascriptParser, expr: &swc_core::ecma::ast::ThisExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(evaluate_identifier(parser: &mut JavascriptParser, for_name: &str, start: u32, end: u32), for_name -> BasicEvaluatedExpression<'static>);
  call_sync_bail_map!(pattern(parser: &mut JavascriptParser, ident: &swc_core::ecma::ast::Ident, for_name: &str), for_name -> bool);
  call_sync_bail_map!(call(parser: &mut JavascriptParser, expr: &CallExpr, for_name: &str), for_name -> bool);
  call_sync_bail_map!(can_rename(parser: &mut JavascriptParser, for_name: &str), for_name -> bool);
  call_sync_bail_map!(rename(parser: &mut JavascriptParser, expr: &Expr, for_name: &str), for_name -> bool);

  pub fn evaluate<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a Expr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate
      .call(parser, expr)
      .expect("evaluate parser hook should not fail")
  }

  pub fn evaluate_call_expression<'a>(
    &self,
    parser: &mut JavascriptParser,
    name: &str,
    expr: &'a CallExpr,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate_call_expression
      .call(parser, name, expr)
      .expect("evaluate_call_expression parser hook should not fail")
  }

  pub fn evaluate_call_expression_member<'a>(
    &self,
    parser: &mut JavascriptParser,
    property: &str,
    expr: &'a CallExpr,
    param: BasicEvaluatedExpression<'a>,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate_call_expression_member
      .call(parser, property, expr, param)
      .expect("evaluate_call_expression_member parser hook should not fail")
  }

  pub fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    self
      .evaluate_typeof
      .get(for_name)
      .and_then(|hook| {
        hook
          .call(parser, expr, for_name)
          .expect("evaluate_typeof parser hook should not fail")
      })
      .or_else(|| {
        self
          .evaluate_typeof_any
          .call(parser, expr, for_name)
          .expect("evaluate_typeof_any parser hook should not fail")
      })
  }
}

impl JavascriptParserHooks {
  pub fn call_member_chain_from_call_expr(
    &self,
    parser: &mut JavascriptParser,
    expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    members_optionals: &[bool],
    member_ranges: &[Span],
  ) -> Option<bool> {
    assert!(matches!(expr.callee, Callee::Expr(_)));
    self.call_member_chain(
      parser,
      expr,
      for_name,
      members,
      members_optionals,
      member_ranges,
    )
  }
}

#[cfg(test)]
mod tests {
  use super::HookMap;
  use crate::parser_plugin::hook::{define_parser_sync_bail_hook, define_parser_sync_hook};

  define_parser_sync_hook!(TestSync, TestSyncHook, (values: &mut Vec<u8>));
  define_parser_sync_bail_hook!(TestBail, TestBailHook, (values: &mut Vec<u8>) -> u8);

  struct Push(u8);

  impl TestSync for Push {
    fn run(&self, values: &mut Vec<u8>) -> rspack_error::Result<()> {
      values.push(self.0);
      Ok(())
    }
  }

  struct MaybeStop {
    value: u8,
    stop: bool,
  }

  impl TestBail for MaybeStop {
    fn run(&self, values: &mut Vec<u8>) -> rspack_error::Result<Option<u8>> {
      values.push(self.value);
      Ok(self.stop.then_some(self.value))
    }
  }

  #[test]
  fn parser_sync_hook_runs_in_registration_order() {
    let mut hook = TestSyncHook::default();
    let mut values = Vec::new();

    hook.tap(Push(1));
    hook.tap(Push(2));
    hook.tap(Push(3));

    hook.call(&mut values).unwrap();

    assert_eq!(values, vec![1, 2, 3]);
  }

  #[test]
  fn parser_sync_bail_hook_stops_at_first_hit() {
    let mut hook = TestBailHook::default();
    let mut values = Vec::new();

    hook.tap(MaybeStop {
      value: 1,
      stop: false,
    });
    hook.tap(MaybeStop {
      value: 2,
      stop: true,
    });
    hook.tap(MaybeStop {
      value: 3,
      stop: false,
    });

    assert_eq!(hook.call(&mut values).unwrap(), Some(2));
    assert_eq!(values, vec![1, 2]);
  }

  #[test]
  fn parser_hook_map_executes_only_matching_key() {
    let mut map = HookMap::<TestBailHook>::default();
    let mut alpha = Vec::new();
    let mut beta = Vec::new();

    map.r#for("alpha").tap(MaybeStop {
      value: 1,
      stop: true,
    });
    map.r#for("beta").tap(MaybeStop {
      value: 2,
      stop: true,
    });

    assert_eq!(map.get("alpha").unwrap().call(&mut alpha).unwrap(), Some(1));
    assert_eq!(alpha, vec![1]);
    assert_eq!(map.get("beta").unwrap().call(&mut beta).unwrap(), Some(2));
    assert_eq!(beta, vec![2]);
    assert!(map.get("gamma").is_none());
  }
}
