# Hyper Line
## Overview
This project is just an experiment to see possible ways to load middleware handlers for processing http1/2 traffic. This is by no means a production ready library, use at your own risk!

* * *

## Project Goals

### Goals Summary
The ideal end-goal for this library is to be able to map endpoints to different handler chains. 
Handlers would be loaded on startup through configuration only (Or Rust equivalent since there is no introspection). Handlers are divided into 2 groups for handling either requests or responses. 
They should also have the option to create request/response completion listeners, and they should have the ability to transfer data between handlers if needed.

### Goals Checklist
- [x] Request/Response Handlers
- [x] Request/Response Complete Listeners
- [ ] Load By Configuration
- [x] Handle TLS 1.1/1.2/1.3
- [ ] Common Logging Patterns
- [ ] Packet Debugger Options
- [ ] Externalize TLS Configuration
- [ ] Protobuf Support
- [ ] Address Endless todo!() Statements

## Outstanding Tasks
- [ ] Allow reverse proxy handler to be non-tls

* * *

## Feature List
TODO

* * *

## How to Setup
TODO

* * *

## How to Contribute
TODO

* * *

## Configuration Guide
TODO

* * *