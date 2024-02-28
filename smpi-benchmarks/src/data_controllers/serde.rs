use crate::data_controllers::Progress;
use safe_mpi::{Result, Tag};
use serde::{de::DeserializeOwned, Serialize};

pub trait SerdeController {
    type Scope: SerdeScope;

    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned;

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned;

    fn scope<F, R>(&self, f: F) -> R
    where
        // F: for<'scope> FnOnce(&'scope Self::Scope<'scope, 'env>) -> R,
        F: FnOnce(&mut Self::Scope) -> R;
}

pub trait SerdeScope: Progress {
    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<<Self as Progress>::Request>
    where
        T: Serialize + DeserializeOwned;

    fn irecv(&mut self, tag: Tag) -> Result<<Self as Progress>::Request>;

    fn data<T>(&self, req: <Self as Progress>::Request) -> Option<T>
    where
        T: Serialize + DeserializeOwned;
}
