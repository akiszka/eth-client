# eth-client

This is a work in progress implementation of an Ethereum client in Rust. After working with EVM smart contracts, I wanted to learn more about the Ethereum protocol and how it works. I decided to implement a client in Rust to learn more about the language and the protocol.

I'm starting from scratch, so many parts of this implementation are not optimized and are not production ready. Especially ECDSA signing and verification is slow and hasn't been checked for correctness.

Implement:

- [x] ecdsa
- [x] rlp
- [ ] merkle trees
- [ ] JSON RPC
- [ ] bloom filters
- [ ] executing smart contracts
- [ ] performing transactions
- [ ] persistent storage
- [ ] communication with other nodes (as consensus client)