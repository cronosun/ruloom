# Rust-Loom (ruloom)

![status](https://img.shields.io/github/actions/workflow/status/cronosun/ruloom/rust.yml)

It is a Crate, which is intended to simplify asynchronous programming. You can write asynchronous code as if it 
were synchronous code. The name is a combination of "Rust" (ru) and "Loom"
(see [https://wiki.openjdk.org/display/loom/Main](https://wiki.openjdk.org/display/loom/Main)). 

# Documentation

[https://docs.rs/ruloom/latest/ruloom/](https://docs.rs/ruloom/latest/ruloom/)

# Why

I love Rust, but am unhappy about how asynchronous programming is solved in Rust (currently). Although there have
been big improvements recently (2024) (see
[https://rust-lang.github.io/rfcs/3425-return-position-impl-trait-in-traits.html](https://rust-lang.github.io/rfcs/3425-return-position-impl-trait-in-traits.html)),
the situation in Rust is still not satisfactory (my opinion).

 * Function coloring.
 * Dependence on a runtime (tokio, smol, ...).
 * Duplication of the standard library and other libraries: Example: Some libraries are sync, some async.
   RwLock, channels, file operations, network operations are available in 2 versions (sync and async).

In my opinion, Rust should have taken a different path:

 * Async/await (like today): In addition, the std-lib should have been prepared for this; at least the traits
   implemented by a runtime like Tokio or Smol should be available in the std-lib, so that libraries do not depend
   on a specific runtime.
 * In addition, Rust should support stackful coroutines (aka Fibers, similar to Go routines or Loom in Java).
   The std-lib should be prepared for this (can be switched on with a feature flag). 

# What does `ruloom` solve?

Ruloom is based on [corosensei](https://crates.io/crates/corosensei) and allows asynchronous code to be written as if it were synchronous code.

```rust
// See, no 'async' keyword here.
// This is how you you write your application / library.
fn looks_like_a_sync_function_but_its_async() {
    // A future
    let future = smol::Timer::after(Duration::from_millis(10));
    // Await the future (suspends the current task)
    await_future(future); 
}

// You can the use smol or tokio to run your code / library / application.
fn run()  {
    smol::block_on(async {
        // convert back to a future.
        let future = to_future(||looks_like_a_sync_function_but_its_async());
        future.await;
    })
}
```

The two main functions of `ruloom` are:

 * `await_future`: Converts a future into a synchronous call. May only be used within
   `to_future`. Technical detail: If the future is pending, the current coroutine is suspended (stack switch).
 * `to_future`: Converts a synchronous function into a future. This future can then be executed
   by the selected runtime (Tokio, Smol, ...).

# What problems does `ruloom` not solve?

It still does not offer a std-library, you still have a dependency on a certain runtime (like Tokio or Smol). 

Furthermore, you have to be careful not to call any blocking functions.

Also, `ru-rust` is (currently) an experiment; it is certainly not advisable to use this code in production.

# Credits

 * 99.9% of the hard work is done by [corosensei](https://crates.io/crates/corosensei), `ruloom` is just a thin wrapper around this library.

# License

Licensed under either of:

Apache License, Version 2.0, (LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or https://opensource.org/licenses/MIT)
at your option.