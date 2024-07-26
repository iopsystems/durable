#![allow(unused)]
use super::types::*;
use fp_bindgen_support::{
    common::{abi::WasmAbi, mem::FatPtr},
    wasmer2_host::{
        errors::{InvocationError, RuntimeError},
        mem::{
            deserialize_from_slice, export_to_guest, export_to_guest_raw, import_from_guest,
            import_from_guest_raw, serialize_to_vec,
        },
        r#async::{create_future_value, future::ModuleRawFuture, resolve_async_value},
        runtime::RuntimeInstanceData,
    },
};
use std::cell::RefCell;
use wasmer::{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv};

#[derive(Clone)]
pub struct Runtime {
    instance: Instance,
    env: RuntimeInstanceData,
}

impl Runtime {
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(module.store(), &env);
        let instance = Instance::new(&module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        Ok(Self { instance, env })
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn default_store() -> wasmer::Store {
        let compiler = wasmer::Cranelift::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn default_store() -> wasmer::Store {
        let compiler = wasmer::Singlepass::default();
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
        "fp" => {
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
            "__fp_gen_abort" => Function::new_native_with_env(store, env.clone(), _abort),
            "__fp_gen_print" => Function::new_native_with_env(store, env.clone(), _print),
            "__fp_gen_task_data" => Function::new_native_with_env(store, env.clone(), _task_data),
            "__fp_gen_task_name" => Function::new_native_with_env(store, env.clone(), _task_name),
            "__fp_gen_transaction_enter" => Function::new_native_with_env(store, env.clone(), _transaction_enter),
            "__fp_gen_transaction_exit" => Function::new_native_with_env(store, env.clone(), _transaction_exit),
        }
    }
}

pub fn _abort(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);
    super::abort(message)
}

pub fn _print(env: &RuntimeInstanceData, data: FatPtr) {
    let data = import_from_guest::<String>(env, data);
    super::print(data)
}

pub fn _task_data(env: &RuntimeInstanceData) -> FatPtr {
    export_to_guest(env, &super::task_data())
}

pub fn _task_name(env: &RuntimeInstanceData) -> FatPtr {
    export_to_guest(env, &super::task_name())
}

pub fn _transaction_enter(
    env: &RuntimeInstanceData,
    label: FatPtr,
    is_db: <bool as WasmAbi>::AbiType,
) -> FatPtr {
    let label = import_from_guest::<String>(env, label);
    let is_db = WasmAbi::from_abi(is_db);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = super::transaction_enter(label, is_db).await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    });
    async_ptr
}

pub fn _transaction_exit(env: &RuntimeInstanceData, data: FatPtr) -> FatPtr {
    let data = import_from_guest::<String>(env, data);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = super::transaction_exit(data).await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    });
    async_ptr
}
