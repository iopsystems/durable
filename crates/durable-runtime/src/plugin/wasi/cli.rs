use wasmtime::component::Resource;

use crate::bindings::wasi;
use crate::bindings::wasi::io::streams::{InputStream, OutputStream};
use crate::error::TaskStatus;
use crate::task::Task;

impl wasi::cli::environment::Host for Task {
    fn get_arguments(&mut self) -> wasmtime::Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn get_environment(&mut self) -> wasmtime::Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }

    fn initial_cwd(&mut self) -> wasmtime::Result<Option<String>> {
        Ok(None)
    }
}

impl wasi::cli::exit::Host for Task {
    fn exit(&mut self, status: Result<(), ()>) -> wasmtime::Result<()> {
        Err(match status {
            Ok(()) => anyhow::Error::new(TaskStatus::ExitSuccess),
            Err(()) => anyhow::Error::new(TaskStatus::ExitFailure),
        })
    }
}

impl wasi::cli::stdin::Host for Task {
    fn get_stdin(&mut self) -> wasmtime::Result<Resource<InputStream>> {
        Ok(Resource::new_own(0))
    }
}

impl wasi::cli::stdout::Host for Task {
    fn get_stdout(&mut self) -> wasmtime::Result<Resource<OutputStream>> {
        Ok(Resource::new_own(1))
    }
}

impl wasi::cli::stderr::Host for Task {
    fn get_stderr(&mut self) -> wasmtime::Result<Resource<OutputStream>> {
        Ok(Resource::new_own(1))
    }
}
