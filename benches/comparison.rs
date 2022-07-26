#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(test)]

#[macro_use]
extern crate async_trait_proto;
#[macro_use]
extern crate async_trait;

extern crate test;

use core::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use test::Bencher;
use test::black_box;

struct SampleFuture {
    yielded: bool,
}

impl SampleFuture {
    fn new() -> Self {
        Self { yielded: false }
    }
}

impl Future for SampleFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            return Poll::Ready(());
        }
        self.yielded = true;
        let waker = cx.waker();
        waker.wake_by_ref();
        Poll::Pending
    }
}

#[derive(Debug, Copy, Clone)]
struct SampleStruct {
    num: u8,
}

impl SampleStruct {
    fn new(num: u8) -> Self {
        Self { num }
    }
}

#[async_trait_proto]
trait FastTrait {
    async fn call_sample(&self) -> usize;
}

macro_rules! trait_impl_content {
    ($self:ident) => {
        for _ in 0..$self.num {
            let fut = SampleFuture::new();
            fut.await;
        }
        tokio::task::yield_now().await;
        return $self.num as usize;
    };
}

#[async_trait_proto]
impl FastTrait for SampleStruct {
    async fn call_sample(&self) -> usize {
        trait_impl_content!(self);
    }
}

#[async_trait]
trait DynTrait {
    async fn call_sample(&self) -> usize;
}

#[async_trait]
impl DynTrait for SampleStruct {
    async fn call_sample(&self) -> usize {
        trait_impl_content!(self);
    }
}

#[inline(never)]
async fn dyn_test<T: DynTrait>(inp: &[T]) {
    for e in inp {
        e.call_sample().await;
    }
}

#[inline(never)]
async fn fast_test<T: FastTrait>(inp: &[T]) {
    for e in inp {
        e.call_sample().await;
    }
}

#[bench]
fn bench_dyn(b: &mut Bencher) {
    let mut i = 100;
    let all_inputs = [0u8; 32]
        .map(|e| {
            i += 1;
            e + i
        })
        .map(SampleStruct::new);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    b.iter(|| {
       runtime.block_on(dyn_test(black_box(&all_inputs[..])));
    });
}

#[bench]
fn bench_fast(b: &mut Bencher) {
    let mut i = 100;
    let all_inputs = [0u8; 32]
        .map(|e| {
            i += 1;
            e + 1
        })
        .map(SampleStruct::new);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    b.iter(|| {
        runtime.block_on(fast_test(black_box(&all_inputs[..])));
    });
}

