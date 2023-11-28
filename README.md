<p align="center">
<a href="https://github.com/kafonek/kernel-sidecar-rs/actions/workflows/ci.yaml">
    <img src="https://github.com/kafonek/kernel-sidecar-rs/actions/workflows/ci.yaml/badge.svg" alt="Tests" />
</a>
</p>

# Overview

This is an attempt to port [kernel-sidecar](https://github.com/kafonek/kernel-sidecar) to Rust.

Roadmap:
 - [x] Establish ZMQ connections with a kernel from information in a connection file
 - [x] Model the high-level Jupyter message spec and wire protocol, including hmac signing
 - [x] Implement the Action and message delegation concepts from Python version
 - [ ] Model all Jupyter messages
 - [ ] Model in-memory Notebook and create utility handlers for updating it based on outputs
 - [ ] Implement CLI similar to Python kernel-sidecar
 - [ ] Create example app of controlling Notebooks from external calls, e.g. AI function calling
 - [ ] Create example of integration with Carabiner GPT
 - [ ] Connect to Jupyter server to stay synced with other users via y-crdt deltas


# Run

1. Start an IPython kernel `python -m ipykernel_launcher --debug -f /tmp/kernel.json`
2. Run `cargo main` (currently hardcoded to read in connection info from `/tmp/kernel.json`)

The `main.rs` script at this time creates an `Action` for a `kernel_info_request` and awaits its completion, meaning it should run until kernel goes to idle and we see `kernel_info_reply`. A `Handler` is attached to the action to `dbg!` all messages related to the original request.