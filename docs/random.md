# Ideas

* **purple haze** (or just **haze** for short) - userland tool to inspect kernel objects using IPC

# IPC syscalls

* register [protocol]#[identifier]: register a server as the handler for a given protocol
  * open question -> do we need multiple services for the same protocol?
* list [protocol]: list all the possible services for a given protocol
* connect [protocol]#[identifier] -> `connect storage#rootfs`, `connect kernel#hendrix`: returns a channel to communicate with the service
* close [channel]
* send [channel] [msg]

Shall we define the common operations for the protocols, such as `open`, `read/get`, `write/put`, `delete`, `close`? 