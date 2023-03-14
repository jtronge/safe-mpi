use serde::{
    Serialize,
    de::DeserializeOwned,
};
use safe_mpi::{
    Result,
    Tag,
};

pub trait SerdeController {
    fn send<T>(&self, data: &T, tag: Tag) -> Result<usize>
    where
        T: Serialize + DeserializeOwned;

    fn recv<T>(&self, tag: Tag) -> Result<T>
    where
        T: Serialize + DeserializeOwned;
}
