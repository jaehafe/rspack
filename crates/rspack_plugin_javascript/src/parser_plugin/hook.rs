use std::borrow::Cow;

use rustc_hash::FxHashMap;
macro_rules! define_parser_sync_hook {
  ($trait_name:ident, $hook_name:ident, ($($arg:ident : $arg_ty:ty),* $(,)?)) => {
    pub trait $trait_name {
      #[allow(clippy::too_many_arguments)]
      fn run(&self, $($arg: $arg_ty),*) -> rspack_error::Result<()>;
    }

    pub struct $hook_name {
      taps: Vec<Box<dyn $trait_name + Send + Sync>>,
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: Vec::new(),
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
      pub fn call(&self, $($arg: $arg_ty),*) -> rspack_error::Result<()> {
        for tap in &self.taps {
          tap.run($($arg),*)?;
        }
        Ok(())
      }

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
      taps: Vec<Box<dyn $trait_name + Send + Sync>>,
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: Vec::new(),
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
      pub fn call(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>> {
        for tap in &self.taps {
          if let Some(result) = tap.run($($arg),*)? {
            return Ok(Some(result));
          }
        }
        Ok(None)
      }

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
      taps: Vec<Box<dyn $trait_name + Send + Sync>>,
    }

    impl Default for $hook_name {
      fn default() -> Self {
        Self {
          taps: Vec::new(),
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
      pub fn call<$lt>(&self, $($arg: $arg_ty),*) -> rspack_error::Result<Option<$ret>> {
        for tap in &self.taps {
          if let Some(result) = tap.run($($arg),*)? {
            return Ok(Some(result));
          }
        }
        Ok(None)
      }

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
  pub fn get(&self, key: &str) -> Option<&H> {
    self.map.get(key)
  }

  #[cfg(test)]
  pub fn len(&self) -> usize {
    self.map.len()
  }
}

impl<H: Default> HookMap<H> {
  pub fn r#for(&mut self, key: impl ToString) -> &mut H {
    self.map.entry(Cow::Owned(key.to_string())).or_default()
  }

  pub fn for_static(&mut self, key: &'static str) -> &mut H {
    self.map.entry(Cow::Borrowed(key)).or_default()
  }
}

#[cfg(test)]
mod tests {
  use super::HookMap;

  #[test]
  fn parser_hook_map_is_lazy() {
    let mut map = HookMap::<Vec<u8>>::default();
    assert_eq!(map.len(), 0);
    assert!(map.get("a").is_none());

    map.for_static("a").push(1);

    assert_eq!(map.len(), 1);
    assert_eq!(map.get("a"), Some(&vec![1]));
  }

  #[test]
  fn parser_hook_map_supports_borrowed_lookup() {
    let mut map = HookMap::<Vec<u8>>::default();
    let key = String::from("alpha");

    map.r#for(&key).extend([1, 2]);

    assert_eq!(map.get("alpha"), Some(&vec![1, 2]));
    assert!(map.get("beta").is_none());
  }
}
