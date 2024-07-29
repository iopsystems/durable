use std::any::Any;

use anymap3::Map;

/// An extension trait for [`Map`] that simplifies some common operations by
/// plugins.
pub trait PluginMapExt {
    /// Returns a reference to the value stored in the collection for the type
    /// `T`.
    ///
    /// # Panics
    /// Panics if there is no value of type `T` stored in the map.
    fn expect<T: Any + Send>(&self) -> &T;

    /// Returns a mutable reference to the value stored in the collection for
    /// the type `T`.
    ///
    /// # Panics
    /// Panics if there is no value of type `T` stored in the map.
    fn expect_mut<T: Any + Send>(&mut self) -> &mut T;
}

impl PluginMapExt for Map<dyn Any + Send> {
    fn expect<T: Any + Send>(&self) -> &T {
        match self.get() {
            Some(value) => value,
            None => panic!(
                "attempted to access type `{}` not contained within the plugin map",
                std::any::type_name::<T>()
            ),
        }
    }

    fn expect_mut<T: Any + Send>(&mut self) -> &mut T {
        match self.get_mut() {
            Some(value) => value,
            None => panic!(
                "attempted to access type `{}` not contained within the plugin map",
                std::any::type_name::<T>()
            ),
        }
    }
}
