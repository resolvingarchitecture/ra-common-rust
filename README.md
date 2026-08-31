# Resolving Architecture Common Library (Rust)

A Rust port of [`ra-common-java`](https://github.com/resolvingarchitecture/ra-common-java) —
the foundational types for the Resolving Architecture / 1M5 ecosystem.

This crate provides:

- **`Envelope`** — the universal message wrapper passed between services.
- **`messaging`** — `Message` (Document / Command / Event / Text), producer/consumer traits.
- **`route`** — routing slips (`DynamicRoutingSlip`) and external/relayed routes.
- **`service`** — the `Service` / `LifeCycle` contract and `ServiceCore` shared state.
- **`identity`** — `DID`, `PublicKey`, `Signature`.
- **`crypto`** — `Hash`, `Multihash`, `HashCash`, password hashing.
- **`content`** — typed content (text / html / json / image / audio / video / binary).
- **`tasks`** — `Task` + a `std::thread`-based `TaskRunner`.
- utilities: base32/58, version comparison, replay `Nonce`, `UniqueId`, byte packing.

Serialization is [`serde`](https://serde.rs)-based and **not** wire-compatible with the Java
version (the Java library used a hand-rolled JSON layer and reflective polymorphism).

## Status

Phase 1 (core). Deferred: currency, locale/i18n, the full network service layer, `Protocol`,
shell/file/browser utilities, `InfoVault`.
