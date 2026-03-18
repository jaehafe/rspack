use std::borrow::Cow;

use rustc_hash::FxHashMap;
macro_rules! define_parser_sync_hook {
  ($trait_name:ident, $hook_name:ident, ($($arg:ident : $arg_ty:ty),* $(,)?)) => {
    pub trait $trait_name {
      #[allow(clippy::too_many_arguments)]
      fn run(&self, $($arg: $arg_ty),*) -> rspack_error::Result<()>;
    }

    pub struct $hook_name {
      taps: smallvec::SmallVec<[Box<dyn $trait_name + Send + Sync>; 1]>,
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: smallvec::SmallVec::new(),
        }
      }
    }

    impl std::fmt::Debug for $hook_name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!($hook_name))
      }
    }

    impl $hook_name {
      #[allow(clippy::too_many_arguments)]
      #[inline]
      pub fn call(&self, $($arg: $arg_ty),*) -> rspack_error::Result<()> {
        for tap in &self.taps {
          tap.run($($arg),*)?;
        }
        Ok(())
      }

      #[inline]
      pub fn tap(&mut self, tap: impl $trait_name + Send + Sync + 'static) {
        self.taps.push(Box::new(tap));
      }
    }
  };
}

macro_rules! define_parser_sync_bail_hook {
  ($trait_name:ident, $hook_name:ident, ($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
    pub trait $trait_name {
      #[allow(clippy::too_many_arguments)]
      fn run(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>>;
    }

    pub struct $hook_name {
      taps: smallvec::SmallVec<[Box<dyn $trait_name + Send + Sync>; 1]>,
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: smallvec::SmallVec::new(),
        }
      }
    }

    impl std::fmt::Debug for $hook_name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!($hook_name))
      }
    }

    impl $hook_name {
      #[allow(clippy::too_many_arguments)]
      #[inline]
      pub fn call(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>> {
        for tap in &self.taps {
          if let Some(result) = tap.run($($arg),*)? {
            return Ok(Some(result));
          }
        }
        Ok(None)
      }

      #[inline]
      pub fn tap(&mut self, tap: impl $trait_name + Send + Sync + 'static) {
        self.taps.push(Box::new(tap));
      }
    }
  };
  ($trait_name:ident, $hook_name:ident, <$lt:lifetime>, ($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
    pub trait $trait_name {
      #[allow(clippy::too_many_arguments)]
      fn run<$lt>(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>>;
    }

    pub struct $hook_name {
      taps: smallvec::SmallVec<[Box<dyn $trait_name + Send + Sync>; 1]>,
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: smallvec::SmallVec::new(),
        }
      }
    }

    impl std::fmt::Debug for $hook_name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, stringify!($hook_name))
      }
    }

    impl $hook_name {
      #[allow(clippy::too_many_arguments)]
      #[inline]
      pub fn call<$lt>(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>> {
        for tap in &self.taps {
          if let Some(result) = tap.run($($arg),*)? {
            return Ok(Some(result));
          }
        }
        Ok(None)
      }

      #[inline]
      pub fn tap(&mut self, tap: impl $trait_name + Send + Sync + 'static) {
        self.taps.push(Box::new(tap));
      }
    }
  };
}

pub(crate) use define_parser_sync_bail_hook;
pub(crate) use define_parser_sync_hook;

#[derive(Debug, Default)]
pub struct HookMap<H> {
  map: FxHashMap<Cow<'static, str>, H>,
}

impl<H> HookMap<H> {
  #[inline]
  pub fn get(&self, key: &str) -> Option<&H> {
    self.map.get(key)
  }
}

impl<H: Default> HookMap<H> {
  #[inline]
  pub fn reserve(&mut self, additional: usize) {
    self.map.reserve(additional);
  }

  #[inline]
  pub fn r#for(&mut self, key: impl ToString) -> &mut H {
    self.map.entry(Cow::Owned(key.to_string())).or_default()
  }

  #[inline]
  pub fn for_static(&mut self, key: &'static str) -> &mut H {
    self.map.entry(Cow::Borrowed(key)).or_default()
  }
}
