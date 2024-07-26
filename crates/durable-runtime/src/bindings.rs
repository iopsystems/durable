#![allow(unused_mut)]

wasmtime::component::bindgen!({
    path: "../durable/wit",
    world: "durable:core/core",
    // tracing: true,
    trappable_imports: true,
    async: {
        except_imports: [
            "task-name",
            "task-data",
            "abort"
        ]
    },
});

/*
use async_trait::async_trait;

/// Auto-generated bindings for a pre-instantiated version of a
/// copmonent which implements the world `core`.
///
/// This structure is created through [`CorePre::new`] which
/// takes a [`InstancePre`](wasmtime::component::InstancePre) that
/// has been created through a [`Linker`](wasmtime::component::Linker).
pub struct CorePre<T> {
    instance_pre: wasmtime::component::InstancePre<T>,
}
impl<T> Clone for CorePre<T> {
    fn clone(&self) -> Self {
        Self {
            instance_pre: self.instance_pre.clone(),
        }
    }
}
/// Auto-generated bindings for an instance a component which
/// implements the world `core`.
///
/// This structure is created through either
/// [`Core::instantiate_async`] or by first creating
/// a [`CorePre`] followed by using
/// [`CorePre::instantiate_async`].
pub struct Core {}

#[async_trait]
pub trait CoreImports: Send {
    /// Get access to the task name.
    fn task_name(&mut self) -> wasmtime::Result<wasmtime::component::__internal::String>;
    /// Get access to the encoded json data
    fn task_data(&mut self) -> wasmtime::Result<wasmtime::component::__internal::String>;
    /// Immediately abort the task with an error.
    fn abort(&mut self, message: wasmtime::component::__internal::String) -> wasmtime::Result<()>;
    /// Start a transaction. If this transaction has already executed to
    /// completion then return the data from the last time it was executed.
    ///
    /// Parameters:
    /// - `label` - A text label that gets recorded in the event. This is used
    ///   to
    /// validate that events are in fact executing in the same order
    /// when the workflow is restarted.
    /// - `is-db` - Whether this transaction is a database transaction and
    ///   should
    /// reserve a database connection so that sql can be used within.
    async fn transaction_enter(
        &mut self,
        label: String,
        is_db: bool,
    ) -> wasmtime::Result<Option<String>>;

    /// Complete a transaction, saving the result of this transaction for future
    /// use.
    ///
    /// Parameters:
    /// - `data` - JSON-encoded state to save.
    async fn transaction_exit(&mut self, data: String) -> wasmtime::Result<()>;

    /// Impure functions.
    ///
    /// It is only valid to call these when within a transaction. Attempting to
    /// call them otherwise will immediately abort the workflow.
    async fn print(&mut self, data: String) -> wasmtime::Result<()>;
}

pub trait CoreImportsGetHost<T>:
    Fn(T) -> <Self as CoreImportsGetHost<T>>::Host + Send + Sync + Copy + 'static
{
    type Host: CoreImports;
}

impl<F, T, O> CoreImportsGetHost<T> for F
where
    F: Fn(T) -> O + Send + Sync + Copy + 'static,
    O: CoreImports,
{
    type Host = O;
}

const _: () = {
    #[allow(unused_imports)]
    use wasmtime::component::__internal::anyhow;
    impl<_T> CorePre<_T> {
        /// Creates a new copy of `CorePre` bindings which can then
        /// be used to instantiate into a particular store.
        ///
        /// This method may fail if the compoennt behind `instance_pre`
        /// does not have the required exports.
        pub fn new(instance_pre: wasmtime::component::InstancePre<_T>) -> wasmtime::Result<Self> {
            let _component = instance_pre.component();
            Ok(CorePre { instance_pre })
        }
        /// Instantiates a new instance of [`Core`] within the
        /// `store` provided.
        ///
        /// This function will use `self` as the pre-instantiated
        /// instance to perform instantiation. Afterwards the preloaded
        /// indices in `self` are used to lookup all exports on the
        /// resulting instance.
        pub async fn instantiate_async(
            &self,
            mut store: impl wasmtime::AsContextMut<Data = _T>,
        ) -> wasmtime::Result<Core>
        where
            _T: Send,
        {
            let mut store = store.as_context_mut();
            let _instance = self.instance_pre.instantiate_async(&mut store).await?;
            Ok(Core {})
        }
        pub fn engine(&self) -> &wasmtime::Engine {
            self.instance_pre.engine()
        }
        pub fn instance_pre(&self) -> &wasmtime::component::InstancePre<_T> {
            &self.instance_pre
        }
    }
    impl Core {
        /// Convenience wrapper around [`CorePre::new`] and
        /// [`CorePre::instantiate_async`].
        pub async fn instantiate_async<_T>(
            mut store: impl wasmtime::AsContextMut<Data = _T>,
            component: &wasmtime::component::Component,
            linker: &wasmtime::component::Linker<_T>,
        ) -> wasmtime::Result<Core>
        where
            _T: Send,
        {
            let pre = linker.instantiate_pre(component)?;
            CorePre::new(pre)?.instantiate_async(store).await
        }
        pub fn add_to_linker_imports_get_host<T>(
            linker: &mut wasmtime::component::Linker<T>,
            host_getter: impl for<'a> CoreImportsGetHost<&'a mut T>,
        ) -> wasmtime::Result<()>
        where
            T: Send,
        {
            let mut linker = linker.root();
            linker.func_wrap(
                "task-name",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                    let host = &mut host_getter(caller.data_mut());
                    let r = CoreImports::task_name(host);
                    Ok((r?,))
                },
            )?;
            linker.func_wrap(
                "task-data",
                move |mut caller: wasmtime::StoreContextMut<'_, T>, (): ()| {
                    let host = &mut host_getter(caller.data_mut());
                    let r = CoreImports::task_data(host);
                    Ok((r?,))
                },
            )?;
            linker.func_wrap(
                "abort",
                move |mut caller: wasmtime::StoreContextMut<'_, T>,
                      (arg0,): (wasmtime::component::__internal::String,)| {
                    let host = &mut host_getter(caller.data_mut());
                    let r = CoreImports::abort(host, arg0);
                    r
                },
            )?;
            linker
                .func_wrap_async(
                    "transaction-enter",
                    move |
                        mut caller: wasmtime::StoreContextMut<'_, T>,
                        (
                            arg0,
                            arg1,
                        ): (wasmtime::component::__internal::String, bool)|
                    wasmtime::component::__internal::Box::new(async move {
                        let host = &mut host_getter(caller.data_mut());
                        let r = CoreImports::transaction_enter(host, arg0, arg1)
                            .await;
                        Ok((r?,))
                    }),
                )?;
            linker.func_wrap_async(
                "transaction-exit",
                move |mut caller: wasmtime::StoreContextMut<'_, T>,
                      (arg0,): (wasmtime::component::__internal::String,)| {
                    wasmtime::component::__internal::Box::new(async move {
                        let host = &mut host_getter(caller.data_mut());
                        let r = CoreImports::transaction_exit(host, arg0).await;
                        r
                    })
                },
            )?;
            linker.func_wrap_async(
                "print",
                move |mut caller: wasmtime::StoreContextMut<'_, T>,
                      (arg0,): (wasmtime::component::__internal::String,)| {
                    wasmtime::component::__internal::Box::new(async move {
                        let host = &mut host_getter(caller.data_mut());
                        let r = CoreImports::print(host, arg0).await;
                        r
                    })
                },
            )?;
            Ok(())
        }
        pub fn add_to_linker<T, U>(
            linker: &mut wasmtime::component::Linker<T>,
            get: impl Fn(&mut T) -> &mut U + Send + Sync + Copy + 'static,
        ) -> wasmtime::Result<()>
        where
            T: Send,
            U: CoreImports + Send,
        {
            Self::add_to_linker_imports_get_host(linker, get)?;
            Ok(())
        }
    }
};
const _: &str =
    "package durable:core@1.0.0;\n\nworld core {\n    // Get access to the task name.\n    import \
     task-name: func() -> string;\n    // Get access to the encoded json data\n    import \
     task-data: func() -> string;\n\n    // Immediately abort the task with an error.\n    import \
     abort: func(message: string);\n\n    // Start a transaction. If this transaction has already \
     executed to completion\n    // then return the data from the last time it was executed.\n    \
     //\n    // Parameters:\n    // - `label` - A text label that gets recorded in the event. \
     This is used to\n    //             validate that events are in fact executing in the same \
     order\n    //             when the workflow is restarted.\n    // - `is-db` - Whether this \
     transaction is a database transaction and should\n    //             reserve a database \
     connection so that sql can be used within.\n    import transaction-enter: func(label: \
     string, is-db: bool) -> option<string>;\n\n    // Complete a transaction, saving the result \
     of this transaction for future use.\n    //\n    // Parameters:\n    // - `data` - \
     JSON-encoded state to save.\n    import transaction-exit: func(data: string);\n\n    // \
     Impure functions.\n    //\n    // It is only valid to call these when within a transaction. \
     Attempting to\n    // call them otherwise will immediately abort the workflow.\n    import \
     print: func(data: string);\n}\n";
*/
