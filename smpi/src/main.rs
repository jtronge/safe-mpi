trait Message {}

trait MessageRequest<'a> {}

trait Messenger {
    type Request: for<'a> MessageRequest<'a>;

    fn isend<'a, M>(&mut self, data: &'a M) -> Self::Request where M: Message;
}

fn main() {}
