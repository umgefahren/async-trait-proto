# async_trait_proto

[![Crates.io](https://img.shields.io/crates/v/async_trait_proto)](https://crates.io/crates/async_trait_proto)
[![docs.rs](https://img.shields.io/docsrs/async_trait_proto)](https://docs.rs/async_trait_proto/latest/async_trait_proto/)
[![Benchmark](https://github.com/umgefahren/async-trait-proto/actions/workflows/benchmark.yml/badge.svg)](https://umgefahren.github.io/async-trait-proto/dev/bench/)
[![Rust](https://github.com/umgefahren/async-trait-proto/actions/workflows/rust.yml/badge.svg)](https://github.com/umgefahren/async-trait-proto/actions/workflows/rust.yml)

Async trait prototype using the desugarization described in [RFC 3185 Static Async Fn in Traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html#equivalent-desugaring).

It should be faster than [async-trait](https://crates.io/crates/async-trait) because it doesn't use allocations on every invocation and type erasure.

[Benchmark](https://umgefahren.github.io/async-trait-proto/dev/bench/)

Requires these feature flags and a **nightly compiler**:
- `#![feature(generic_associated_types)]`
- `#![feature(type_alias_impl_trait)]`

### Example
```rust
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
use async_trait_proto::async_trait_proto;
struct Foo;

#[async_trait_proto]
trait Bar {
    async fn wait(&self);
}

#[async_trait_proto]
impl Bar for Foo {
    async fn wait(&self) {
        sleep(Duration::from_secs(10)).await;
    }
}
```

License: Unlicense
