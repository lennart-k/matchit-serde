# matchit-serde

Provides a serde deserializer for matchit params.

The behaviour is probably not always identical to axum's path matching, however for simple data structures and data-types it should work.

Some code sections are either copied from or heavily inspired by [axum](https://github.com/tokio-rs/axum/blob/main/axum/src/extract/path).
This crate is neither affiliated with axum nor matchit.
