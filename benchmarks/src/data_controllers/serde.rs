use serde::{
    Serialize,
    de::DeserializeOwned,
};
use safe_mpi::{
    Result,
    Tag,
};

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

pub trait SerdeScope {
    type Request: Copy;

    fn isend<T>(&mut self, data: &T, tag: Tag) -> Result<Self::Request>
    where
        T: Serialize + DeserializeOwned;

    fn irecv(&mut self, tag: Tag) -> Result<Self::Request>;

    fn progress(&mut self, req: Self::Request) -> Result<SerdeRequestStatus>;

    fn data<T>(&self, req: Self::Request) -> Option<T> where T: Serialize + DeserializeOwned;

    fn wait_all(&mut self, reqs: &[Self::Request]) -> Result<()> {
        loop {
            let mut not_done = 0;
            for req in reqs {
                for i in 0..16 {
                    match self.progress(*req)? {
                        SerdeRequestStatus::InProgress => not_done += 1,
                        SerdeRequestStatus::Complete => (),
                    }
                }
            }
            if not_done == 0 {
                break;
            }
        }
        Ok(())
    }
}

pub enum SerdeRequestStatus {
    InProgress,
    Complete,
}
