// 演算時要素の列挙型
#[derive(Debug)]
pub enum Expr {
    Numbers(f64),
    Binomial(BinomialFunc),
    Monomial(MonomialFunc),
    Const(Constant),
    Opstack(OperateStack),
}
// 二項演算の列挙型
#[derive(Debug)]
pub enum BinomialFunc {
    Add,
    Subtract,
    Multiply,
    Divide,
    Pow,
    Log,
}

// 単項演算の列挙型
#[derive(Debug)]
pub enum MonomialFunc {
    Sqrt,
    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    ATan,
    ToDeg,
    ToRad,
}

// 定数の列挙型
#[derive(Debug)]
pub enum Constant {
    Pi,
    E,
}

// スタック操作の列挙型
#[derive(Debug)]
pub enum OperateStack {
    Swap,
    Clear,
    Sum,
    Deg,
    Rad,
}

// 角度モードの列挙型
#[derive(Debug)]
pub enum DegMode {
    Rad,
    Deg,
}
