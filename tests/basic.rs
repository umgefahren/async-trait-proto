#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use tokio::task::yield_now;

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
        yield_now().await;
    }
    async fn lol(self) {
        yield_now().await;
    }
    fn unasync(&self) {
        std::thread::yield_now();
    }
}

struct Content {
    a: u128,
    b: u128,
}

#[async_trait_proto::async_trait_proto]
impl Interesting for Content {
    async fn interesting(&self) {
        let _ = (self.a + self.b).leading_zeros();
        yield_now().await;
    }

    async fn lol(self) {
        let _ = (self.a + self.b).leading_zeros();
        yield_now().await;
    }

    fn unasync(&self) {
        std::thread::yield_now();
    }
}

async fn test_function<T: Interesting>(obj: T) {
    obj.interesting().await;
    obj.unasync();
    obj.lol().await;
}

#[tokio::test]
async fn empty_impl() {
    let empty = Empty;
    test_function(empty).await;
}

#[tokio::test]
async fn content_impl() {
    let content = Content {
        a: 1923,
        b: 0x922112,
    };
    test_function(content).await;
}