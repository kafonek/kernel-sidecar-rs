# Overview

This is an attempt to port [kernel-sidecar](https://github.com/kafonek/kernel-sidecar) to Rust.

Roadmap:
 - [x] Establish ZMQ connections with a kernel from information in a connection file
 - [x] Model the high-level Jupyter message spec and wire protocol, including hmac signing
 - [ ] Implement the Action and message delegation concepts from Python version
   - (Action concept and message routing is working, but the Action "future" resolution is not done)
 - [ ] Model all Jupyter messages
 - [ ] Model in-memory Notebook and create utility handlers for updating it based on outputs
 - [ ] Implement CLI similar to Python kernel-sidecar
 - [ ] Create example app of controlling Notebooks from external calls, e.g. AI function calling
 - [ ] Create example of integration with Carabiner GPT
 - [ ] Connect to Jupyter server to stay synced with other users via y-crdt deltas
