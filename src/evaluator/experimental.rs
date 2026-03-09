use crate::parser::Stmt;
use crate::evaluator::{Evaluator, ControlFlow};

/// # Experimental Layer
///
/// このモジュールは、実装直後の不安定な機能や、試験的な構文の実行ロジックを分離するための場所です。
/// 安定性が確認された機能は、順次 `control_flow.rs` や `expression.rs` 等の適切な場所へ移動（マージ）されます。
///
/// ## 役割
/// 1. **影響の局所化**: 新機能のバグやパニックが本体（Evaluator）のコアロジックに影響を与えるのを最小限にします。
/// 2. **デバッグの容易化**: 「どのコードが試験的か」を明白にすることで、不具合発生時の調査対象を絞り込みます。
/// 3. **プロトタイピング**: 本体の構造を汚さずに新しいアイディアを素早く試すことができます。

impl Evaluator {
    /// 試験的なステートメントの実行を試みます。
    /// 
    /// 本メソッドは `mod.rs` の `exec_stmt` から呼び出されます。
    /// 試験的な機能としてマッチした場合は `Some(ControlFlow)` を返し、それ以外は `None` を返します。
    pub(crate) fn exec_experimental(&mut self, stmt: &Stmt) -> Option<ControlFlow> {
        match stmt {
            // 例: FOR-IN は比較的新しい機能なため、一旦こちらに配置
            Stmt::ForIn { var, array, body, .. } => {
                Some(self.exec_for_in_experimental(var, array, body))
            }
            
            // 将来追加される試験的機能の例:
            // Stmt::NewFeature { ... } => Some(self.exec_new_feature(...)),

            _ => None,
        }
    }

    /// [EXPERIMENTAL] FOR-IN ループの実装例
    /// 安定したら control_flow.rs へ移動されます。
    fn exec_for_in_experimental(&mut self, var: &String, array: &crate::parser::Expr, body: &[Stmt]) -> ControlFlow {
        let array_val = match self.eval_expr(array) {
            Ok(v) => v,
            Err(e) => return ControlFlow::Error(e),
        };
        
        if let crate::value::RuntimeValue::Array(elements) = array_val {
            for element in elements {
                self.env.lock().unwrap().define(var.clone(), element, false);
                let cf = self.exec_stmts(body);
                match cf {
                    ControlFlow::Break(n) => {
                        if n <= 1 { break; }
                        else { return ControlFlow::Break(n - 1); }
                    }
                    ControlFlow::Continue(n) => {
                        if n <= 1 { continue; }
                        else { return ControlFlow::Continue(n - 1); }
                    }
                    ControlFlow::Return(_) | ControlFlow::Error(_) | ControlFlow::Exit | ControlFlow::ExitExit(_) => return cf,
                    ControlFlow::None => {}
                }
            }
            ControlFlow::None
        } else {
            ControlFlow::Error(format!("FOR-IN requires an array, but got {}", array_val))
        }
    }
}
