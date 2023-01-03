use std::marker::PhantomData;

pub struct Status;

pub struct Request<'a> {
    phantom_data: PhantomData<&'a ()>
}

impl<'a> Request<'a> {
    pub unsafe fn status(&self) -> Status {
        Status
    }
}
