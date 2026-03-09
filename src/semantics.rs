use crate::parser::CmpOp;
use crate::value::RuntimeValue;

/// UWSCの算術演算の契約（セマンティクス）。
/// 参照: ORIGINAL_LANGUAGE_SPEC.md §2.2
pub trait UwscArithmetic {
    /// 加算・連結 (+)
    fn add(&self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 減算 (-)
    fn sub(&self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 乗算 (*)
    fn mul(&self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 除算 (/) - 特殊ルール: ゼロ除算は 0.0 を返す。結果は常に実数。
    fn div(&self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 剰余 (MOD) - 特殊ルール: ゼロ除算は 0 を返す。
    fn mod_op(&self, rhs: &Self) -> Result<Self, String> where Self: Sized;
}

/// UWSCの比較演算の契約。
/// 参照: [ORIGINAL_LANGUAGE_SPEC.md#L27](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC.md#L27) (優先順位)
/// 参照: [ORIGINAL_LANGUAGE_SPEC_JA.md#L20](file:///c:/dev/uwsc/uwsc-rs/docs/ORIGINAL_LANGUAGE_SPEC_JA.md#L20) (大文字小文字の区別)
/// 境界条件: 
/// - 文字列比較は常にケースインセンシティブ。
/// - 数値と（数値に見える）文字列の比較は数値として行われます (§34)。
/// - それ以外の型同士は文字列化して比較されます。
pub trait UwscCompare {
    fn compare(&self, rhs: &Self, op: CmpOp) -> Result<RuntimeValue, String>;
}

/// UWSCの論理/ビット演算の契約。
/// 参照: [DETAILED_LANGUAGE_SPEC_JA.md#L73](file:///c:/dev/uwsc/uwsc-rs/docs/DETAILED_LANGUAGE_SPEC_JA.md#L73) (優先順位/種類)
/// 境界条件:
/// - 数値型 (Integer) 同士の場合はビット演算として振る舞います。
/// - それ以外の場合は論理演算（is_truthyに基づく）として振る舞います。
pub trait UwscLogical {
    fn and(&self, rhs: &Self) -> Result<RuntimeValue, String>;
    fn or(&self, rhs: &Self) -> Result<RuntimeValue, String>;
    fn xor(&self, rhs: &Self) -> Result<RuntimeValue, String>;
}

/// UWSCの単項演算の契約。
/// 参照: [DETAILED_LANGUAGE_SPEC_JA.md#L45](file:///c:/dev/uwsc/uwsc-rs/docs/DETAILED_LANGUAGE_SPEC_JA.md#L45) (UnaryOp)
pub trait UwscUnary {
    /// 論理否定/ビット反転 (!)
    /// 整数にはビット反転、その他には論理否定を適用します。
    fn not(&self) -> Result<RuntimeValue, String>;
    /// 算術反転 (-)
    /// 数値または数値化可能な文字列に適用します (§34)。
    fn neg(&self) -> Result<RuntimeValue, String>;
}

/// UWSCの代入演算の契約。
/// 参照: ORIGINAL_LANGUAGE_SPEC.md §2.2
/// 複合代入は内部的に UwscArithmetic を利用して計算されます。
pub trait UwscAssignment {
    /// 単純代入 (=)
    fn assign(&mut self, source: Self) -> Result<Self, String> where Self: Sized;
    /// 加算代入 (+=)
    fn add_assign(&mut self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 減算代入 (-=)
    fn sub_assign(&mut self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 乗算代入 (*=)
    fn mul_assign(&mut self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 除算代入 (/=)
    fn div_assign(&mut self, rhs: &Self) -> Result<Self, String> where Self: Sized;
    /// 剰余代入 (MOD=)
    fn mod_assign(&mut self, rhs: &Self) -> Result<Self, String> where Self: Sized;
}
