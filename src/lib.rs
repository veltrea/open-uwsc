// lib.rs — ライブラリクレートのエントリポイント
// main.rs と同じモジュールを公開することで、tests/ 以下からもアクセス可能にする。
// これにより src/test_*.rs を tests/unit/ に移動できる。
pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod evaluator;
pub mod builtins;
pub mod value;
pub mod environment;
pub mod resolver;
