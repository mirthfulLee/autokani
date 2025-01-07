# AUTO-KANI

Generate test harness for target function accoding to its signature.
The test harness will be checked by [kani](https://github.com/model-checking/kani).

## Features

The generated harness is capable to catch some common errors, e.g., arithmetic overflow, illegal raw pointer dereference, illegal memory access, run-time panic.

## Usage

**Support for target struct**:
Add `#[kani_arbitrary]` to target struct or add `#[extend_arbitrary]` to the basic impl block of target struct;

> One struct can only deploy one of `#[kani_arbitrary]` or `#[extend_arbitrary]`;
> `#[extend_arbitrary]` is more recommended for less false alarms.

**Run the kani harness**:

1. Add `#[kani_test]` for target function
2. Run `cargo kani --harness check_{function_name}` for specific target or `cargo kani` for all selected functions.

> If the code involves raw pointers, use `cargo kani -Z mem-predicates`.