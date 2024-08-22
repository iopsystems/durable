use std::any::Any;

use anymap3::Map;
use slab::Slab;
use wasmtime::component::Resource;

pub trait Resourceable: 'static {
    const NAME: &'static str;

    type Data: Send + Sync + 'static;
}

struct Entry<T> {
    data: T,
    txn: Option<i32>,
}

struct ResourceSlab<R: Resourceable>(Slab<Entry<R::Data>>);

impl<R: Resourceable> Default for ResourceSlab<R> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Default)]
pub struct Resources {
    txn: Option<i32>,
    data: Map<dyn Any + Send + Sync>,
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_txn(&mut self, txn: Option<i32>) {
        self.txn = txn;
    }

    /// Fetch the data associated with the resource `res`.
    ///
    /// `txn` should be the index of the current transaction, if the workflow is
    /// in one.
    ///
    /// # Errors
    /// This will return an error if:
    /// - there is no resource with the provided resource id,
    /// - the resource was created in a transaction other than the current one.
    pub fn get<R>(&self, res: Resource<R>) -> wasmtime::Result<&R::Data>
    where
        R: Resourceable,
    {
        let Some(slab) = self.data.get::<ResourceSlab<R>>() else {
            anyhow::bail!("no resources present for resource type `{}`", R::NAME);
        };

        let Ok(index) = usize::try_from(res.rep()) else {
            anyhow::bail!("resource index ({}) was larger than usize::MAX", res.rep())
        };

        let entry = &slab.0[index];
        if entry.txn.is_some() && entry.txn != self.txn {
            anyhow::bail!(
                "attempted to use resource of type `{}` outside of the transaction it was created \
                 in",
                R::NAME
            )
        }

        Ok(&entry.data)
    }

    /// Mutably fetch the data associated with the resource `res`.
    ///
    /// `txn` should be the index of the current transaction, if the workflow is
    /// in one.
    ///
    /// # Errors
    /// This will return an error if:
    /// - there is no resource with the provided resource id,
    /// - the resource was created in a transaction other than the current one.
    pub fn get_mut<R>(&mut self, res: Resource<R>) -> wasmtime::Result<&mut R::Data>
    where
        R: Resourceable,
    {
        let Some(slab) = self.data.get_mut::<ResourceSlab<R>>() else {
            anyhow::bail!("no resources present for resource type `{}`", R::NAME);
        };

        let Ok(index) = usize::try_from(res.rep()) else {
            anyhow::bail!("resource index ({}) was larger than usize::MAX", res.rep())
        };

        let entry = &mut slab.0[index];
        if entry.txn.is_some() && entry.txn != self.txn {
            anyhow::bail!(
                "attempted to use resource of type `{}` outside of the transaction it was created \
                 in",
                R::NAME
            )
        }

        Ok(&mut entry.data)
    }

    pub fn get_txn<R>(&mut self, res: Resource<R>) -> wasmtime::Result<Option<i32>>
    where
        R: Resourceable,
    {
        let Some(slab) = self.data.get::<ResourceSlab<R>>() else {
            anyhow::bail!("no resources present for resource type `{}`", R::NAME);
        };

        let Ok(index) = usize::try_from(res.rep()) else {
            anyhow::bail!("resource index ({}) was larger than usize::MAX", res.rep())
        };

        let entry = &slab.0[index];
        if entry.txn.is_some() && entry.txn != self.txn {
            anyhow::bail!(
                "attempted to use resource of type `{}` outside of the transaction it was created \
                 in",
                R::NAME
            )
        }

        Ok(entry.txn)
    }

    pub fn insert<R>(&mut self, data: R::Data) -> wasmtime::Result<Resource<R>>
    where
        R: Resourceable,
    {
        let slab: &mut ResourceSlab<R> = self.data.entry().or_default();
        let index = slab.0.insert(Entry {
            data,
            txn: self.txn,
        });
        let index = match u32::try_from(index) {
            Ok(index) => index,
            Err(_) => {
                slab.0.remove(index);
                anyhow::bail!(
                    "no room left in address space to create resource `{}`",
                    R::NAME
                )
            }
        };

        Ok(Resource::new_own(index))
    }

    pub fn remove<R>(&mut self, res: Resource<R>) -> wasmtime::Result<R::Data>
    where
        R: Resourceable,
    {
        let Some(slab) = self.data.get_mut::<ResourceSlab<R>>() else {
            anyhow::bail!("no resources present for resource type `{}`", R::NAME);
        };

        let Ok(index) = usize::try_from(res.rep()) else {
            anyhow::bail!("resource index ({}) was larger than usize::MAX", res.rep())
        };

        let entry = slab.0.remove(index);
        if entry.txn.is_some() && entry.txn != self.txn {
            anyhow::bail!(
                "attempted to use resource of type `{}` outside of the transaction it was created \
                 in",
                R::NAME
            )
        }

        Ok(entry.data)
    }
}
