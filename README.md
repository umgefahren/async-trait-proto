# async_trait_proto

Async trait prototype using the desugarization described in [RFC 3185 Static Async Fn in Traits](https://rust-lang.github.io/rfcs/3185-static-async-fn-in-trait.html#equivalent-desugaring).

It should be faster than [async-trait](https://crates.io/crates/async-trait) because it doesn't use allocations on every invocation and type erasure.

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
