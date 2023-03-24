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

/*
    fn isend<T>(&self, data: &T, tag: Tag) -> Result<SerdeSendRequest>
    where
        T: Serialize + DeserializeOwned;

    fn irecv<T>(&self, tag: Tag) -> Result<SerdeRecvRequest<T>>
    where
        T: Serialize + DeserializeOwned;

    fn wait_all<R>(&self, requests: &mut [R]) -> Result<()>
    where
        R: SerdeRequest;
*/
}
