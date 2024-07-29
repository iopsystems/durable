use wasmtime::component::Resource;

use crate::bindings::wasi;
use crate::bindings::wasi::filesystem::types::*;
use crate::task::Task;

// It is not possible to get access to a file handle within the workflow so all
// of these are stubs.
#[async_trait::async_trait]
impl wasi::filesystem::types::HostDescriptor for Task {
    async fn read_via_stream(
        &mut self,
        _: Resource<Descriptor>,
        _: Filesize,
    ) -> wasmtime::Result<Result<Resource<InputStream>, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.read-via-stream is not implemented")
    }

    async fn write_via_stream(
        &mut self,
        _: Resource<Descriptor>,
        _: Filesize,
    ) -> wasmtime::Result<Result<Resource<OutputStream>, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.write-via-stream is not implemented")
    }

    async fn append_via_stream(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<Resource<OutputStream>, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.append-via-stream is not implemented")
    }

    async fn advise(
        &mut self,
        _: Resource<Descriptor>,
        _: Filesize,
        _: Filesize,
        _: Advice,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.advise is not implemented")
    }

    async fn sync_data(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.sync-data is not implemented")
    }

    async fn get_flags(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<DescriptorFlags, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.get-flags is not implemented")
    }

    async fn get_type(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<DescriptorType, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.get-type is not implemented")
    }

    async fn set_size(
        &mut self,
        _: Resource<Descriptor>,
        _: Filesize,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.set-size is not implemented")
    }

    async fn set_times(
        &mut self,
        _: Resource<Descriptor>,
        _: NewTimestamp,
        _: NewTimestamp,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.set-times is not implemented")
    }

    async fn read(
        &mut self,
        _: Resource<Descriptor>,
        _: Filesize,
        _: Filesize,
    ) -> wasmtime::Result<Result<(Vec<u8>, bool), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.read is not implemented")
    }

    async fn write(
        &mut self,
        _: Resource<Descriptor>,
        _: Vec<u8>,
        _: Filesize,
    ) -> wasmtime::Result<Result<Filesize, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.write is not implemented")
    }

    async fn read_directory(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<Resource<DirectoryEntryStream>, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.read-directory is not implemented")
    }

    async fn sync(&mut self, _: Resource<Descriptor>) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.sync is not implemented")
    }

    async fn create_directory_at(
        &mut self,
        _: Resource<Descriptor>,
        _: String,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.create-directory-at is not implemented")
    }

    async fn stat(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<DescriptorStat, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.stat is not implemented")
    }

    async fn stat_at(
        &mut self,
        _: Resource<Descriptor>,
        _: PathFlags,
        _: String,
    ) -> wasmtime::Result<Result<DescriptorStat, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.stat-at is not implemented")
    }

    async fn set_times_at(
        &mut self,
        _: Resource<Descriptor>,
        _: PathFlags,
        _: String,
        _: NewTimestamp,
        _: NewTimestamp,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.set-times-at is not implemented")
    }

    async fn link_at(
        &mut self,
        _: Resource<Descriptor>,
        _: PathFlags,
        _: String,
        _: Resource<Descriptor>,
        _: String,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.link-at is not implemented")
    }

    async fn open_at(
        &mut self,
        _: Resource<Descriptor>,
        _: PathFlags,
        _: String,
        _: OpenFlags,
        _: DescriptorFlags,
    ) -> wasmtime::Result<Result<Resource<Descriptor>, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.open-at is not implemented")
    }

    async fn readlink_at(
        &mut self,
        _: Resource<Descriptor>,
        _: String,
    ) -> wasmtime::Result<Result<String, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.readlink-at is not implemented")
    }

    async fn remove_directory_at(
        &mut self,
        _: Resource<Descriptor>,
        _: String,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.remove-directory-at is not implemented")
    }

    async fn rename_at(
        &mut self,
        _: Resource<Descriptor>,
        _: String,
        _: Resource<Descriptor>,
        _: String,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.rename-at is not implemented")
    }

    async fn symlink_at(
        &mut self,
        _: Resource<Descriptor>,
        _: String,
        _: String,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.symlink-at is not implemented")
    }

    async fn unlink_file_at(
        &mut self,
        _: Resource<Descriptor>,
        _: String,
    ) -> wasmtime::Result<Result<(), ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.unlink-file-at is not implemented")
    }

    async fn is_same_object(
        &mut self,
        _: Resource<Descriptor>,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<bool> {
        anyhow::bail!("wasi:filesystem/types.descriptor.is-same-object is not implemented")
    }

    async fn metadata_hash(
        &mut self,
        _: Resource<Descriptor>,
    ) -> wasmtime::Result<Result<MetadataHashValue, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.metadata-hash is not implemented")
    }

    async fn metadata_hash_at(
        &mut self,
        _: Resource<Descriptor>,
        _: PathFlags,
        _: String,
    ) -> wasmtime::Result<Result<MetadataHashValue, ErrorCode>> {
        anyhow::bail!("wasi:filesystem/types.descriptor.metadata-hash-at is not implemented")
    }

    fn drop(&mut self, _: Resource<Descriptor>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl wasi::filesystem::types::HostDirectoryEntryStream for Task {
    async fn read_directory_entry(
        &mut self,
        _: Resource<DirectoryEntryStream>,
    ) -> wasmtime::Result<Result<Option<DirectoryEntry>, ErrorCode>> {
        anyhow::bail!(
            "wasi:filesystem/types.directory-entry-stream.read-directory-entry is not implemented"
        )
    }

    fn drop(&mut self, _: Resource<DirectoryEntryStream>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl wasi::filesystem::types::Host for Task {
    async fn filesystem_error_code(
        &mut self,
        _: Resource<Error>,
    ) -> wasmtime::Result<Option<ErrorCode>> {
        // The entire filesystem module is a stub so there are no fs error codes
        Ok(None)
    }
}

#[async_trait::async_trait]
impl wasi::filesystem::preopens::Host for Task {
    async fn get_directories(&mut self) -> wasmtime::Result<Vec<(Resource<Descriptor>, String)>> {
        Ok(Vec::new())
    }
}
