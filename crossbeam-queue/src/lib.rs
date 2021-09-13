//! Concurrent queues.
//!
//! This crate provides concurrent queues that can be shared among threads:
//!
//! * [`ArrayQueue`], a bounded MPMC queue that allocates a fixed-capacity buffer on construction.
//! * [`SegQueue`], an unbounded MPMC queue that allocates small buffers, segments, on demand.

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(not(feature = "std"), no_std)]

#![cfg_attr(all(not(target_env = "sgx"), feature = "mesalock_sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(not(crossbeam_no_atomic_cas))]
cfg_if::cfg_if! {
    if #[cfg(all(feature = "alloc", not(target_env = "sgx"), not(feature = "mesalock_sgx")))] 
    {
        extern crate alloc;

        mod array_queue;
        mod seg_queue;

        pub use self::array_queue::ArrayQueue;
        pub use self::seg_queue::SegQueue;
    }
    else if #[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))] 
    {
        pub extern crate sgx_tstd as alloc;

        mod array_queue;
        mod seg_queue;

        pub use self::array_queue::ArrayQueue;
        pub use self::seg_queue::SegQueue;
    }
    else if #[cfg(any(target_env = "sgx"))] 
    {
        mod array_queue;
        mod seg_queue;

        pub use self::array_queue::ArrayQueue;
        pub use self::seg_queue::SegQueue;
    }
}
