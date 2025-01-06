# AUTO-KANI
Generate test harness for target function accoding to its signature.
The test harness will be checked by [kani](https://github.com/model-checking/kani).

## Features
The generated harness is capable to catch some common errors, e.g., arithmetic overflow, illegal raw pointer dereference, illegal memory access, run-time panic.