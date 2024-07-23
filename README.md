# Gerevs: A Rust Crate for Secure SOCKS5 Proxies ([RFC 1928](https://datatracker.ietf.org/doc/html/rfc1928))

Welcome to **Gerevs**!

Gerevs is a work-in-progress Rust crate designed to simplify the creation of secure, general-purpose SOCKS5 proxies. Whether you’re building a networked application or enhancing security, Gerevs aims to provide the tools you need with minimal hassle.

## Features

- **Secure Connections**: Ensure robust security for your proxy communications.
- **General Purpose**: Flexible enough to suit a variety of use cases.
- **Rust Power**: Leverage Rust’s performance and safety features.
- **Asynchronous Execution**: Built using Tokio for high performance and efficient asynchronous operations.

## SOCKS5 Commands
- [x] CONNECT
- [x] BIND
- [x] UDP ASSOCIATE (The proxy still doesn't support fragmentation, but I doubt it will because after scouring the internet I couldn't find client side implementations that actually bothered to implement fragmentation)

## SOCKS5 Authentication
- [x] Username password ([RFC 1929](https://datatracker.ietf.org/doc/html/rfc1929))
- [ ] GSSAPI ([RFC 1961](https://www.rfc-editor.org/rfc/rfc1961.html))
- [x] User defined (The library allows the user of the library to define authentication methods themselves)

Note: Gerevs is designed for server-side implementation only.

## What's in the Name?

The name **Gerevs** is derived from the Hebrew word "גרב" (gerev), which means "sock".

## Getting Started

Gerevs is still under active development and not yet available on crates.io. Stay tuned for updates!

## Contributions

We welcome contributions! Check out our [GitHub repository](https://github.com/itamarsch/gerevs) to get involved.

Join us in making proxy development easier and more secure with Rust! 🚀
