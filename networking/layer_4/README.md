# Layer 4
The fourth layer in the OSI stack is responsible for transmitting data segments between nodes within a network. It is considered the "transport" layer.

This means that its various protocols often:
* handle the segmentation (breaking up) of packets from source to destination to ensure transmission
* handle reliability of the connection and provide mechanisms for recovery from packet loss.

Here, we have some writing on two protocols that often get referenced as Layer 4 transport protocols, as defined in the list below:

| Protocol | Description | Use cases |
| :---: | :--- | :--- |
| [tcp](./tcp) | Reliable and ordered delivery of a stream of octets | Bank application client |
| [udp](./udp) | Connectionless and not error checked delivery of a stream of octets | DNS, Video Chat, Latency-sensitive applications |