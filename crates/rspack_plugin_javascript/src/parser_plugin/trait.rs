use std::sync::Arc;

use super::JavascriptParserHooks;
use crate::parser_and_generator::ParserRuntimeRequirementsData;

pub struct JavascriptParserPluginContext<'a> {
  pub hooks: &'a mut JavascriptParserHooks,
  pub parser_runtime_requirements: &'a ParserRuntimeRequirementsData,
}

pub trait JavascriptParserPlugin: Send + Sync {
  fn apply(self: Arc<Self>, context: &mut JavascriptParserPluginContext<'_>);
}

pub type BoxJavascriptParserPlugin = Arc<dyn JavascriptParserPlugin + Send + Sync>;

#[macro_export]
macro_rules! impl_javascript_parser_hook {
  ($plugin:ty, $hook_trait:path, <$lt:lifetime>, $method:ident($($arg:ident : $arg_ty:ty),* $(,)?)) => {
    impl $hook_trait for ::std::sync::Arc<$plugin> {
      fn run<$lt>(&self, $($arg: $arg_ty),*) -> ::rspack_error::Result<()> {
        self.$method($($arg),*);
        Ok(())
      }
    }
  };
  ($plugin:ty, $hook_trait:path, <$lt:lifetime>, $method:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
    impl $hook_trait for ::std::sync::Arc<$plugin> {
      fn run<$lt>(
        &self,
        $($arg: $arg_ty),*
      ) -> ::rspack_error::Result<::std::option::Option<$ret>> {
        Ok(self.$method($($arg),*))
      }
    }
  };
  ($plugin:ty, $hook_trait:path, $method:ident($($arg:ident : $arg_ty:ty),* $(,)?)) => {
    impl $hook_trait for ::std::sync::Arc<$plugin> {
      fn run(&self, $($arg: $arg_ty),*) -> ::rspack_error::Result<()> {
        self.$method($($arg),*);
        Ok(())
      }
    }
  };
  ($plugin:ty, $hook_trait:path, $method:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
    impl $hook_trait for ::std::sync::Arc<$plugin> {
      fn run(&self, $($arg: $arg_ty),*) -> ::rspack_error::Result<::std::option::Option<$ret>> {
        Ok(self.$method($($arg),*))
      }
    }
  };
}
