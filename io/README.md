# Input / Output
Here, we'll discuss operating-system level protocols for dealing with input / output between processes. UNIX system calls like `select` and `poll` exposed the ability to monitor the status of file descriptors (in UNIX everything is a file) and whether they were ready to perform IO operations.

The patterns for these operations exist as the bedrock of many frameworks responsible for managing systems at a higher level. With the advent of modern web-exposed servers, this became an acute problem. The [c10k problem](http://www.kegel.com/c10k.html) discusses this problem in detail.

## `epoll`
`epoll` was introduced in the Linux kernal at version 2.5.44. It serves to solve the c10k problem by providing an event-based facility for monitoring many file descriptors at once. It is discussed in [`epoll`](./epoll).
