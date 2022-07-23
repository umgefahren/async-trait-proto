#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[async_trait_proto::async_trait_proto]
trait Interesting {
    async fn interesting(&self);
    async fn lol(self);
    fn unasync(&self);
}

struct Empty;

#[async_trait_proto::async_trait_proto]
impl Interesting for Empty {
    async fn interesting(&self) {
        todo!()
    }
    async fn lol(self) {
        todo!()
    }
    fn unasync(&self) {
        todo!()
    }
}
