pub fn future_value(n: f64, iyr: f64, pv: f64, pmt: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 各期の支払い額（PMT, Payment）、支払い回数（N, Number of Periods）
    let i_yr = iyr / 100.0 / 12.0;
    let fv = pv * (1.0 + i_yr).powf(n) + pmt * ((1.0 + i_yr).powf(n) - 1.0) / i_yr;
    -fv
}

pub fn payment(n: f64, iyr: f64, pv: f64, fv: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 未来価値（FV, Future Value）、支払い回数（N, Number of Periods）
    let i_yr = iyr / 100.0 / 12.0;
    let pmt = (fv - pv * (1.0 + i_yr).powf(n)) / ((1.0 + i_yr).powf(n) - 1.0) * i_yr;
    -pmt
}
pub fn present_value(n: f64, iyr: f64, pmt: f64, fv: f64) -> f64 {
    // 未来価値（FV, Future Value）、利率（IYR, Interest Rate per Year）
    // 各期の支払い額（PMT, Payment）、支払い回数（N, Number of Periods）
    let i_yr = iyr / 100.0 / 12.0;
    let pv = (fv - pmt * ((1.0 + i_yr).powf(n) - 1.0) / i_yr) / ((1.0 + i_yr).powf(n));
    -pv
}

pub fn number_of_periods(iyr: f64, pv: f64, pmt: f64, fv: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 未来価値（FV, Future Value）、各期の支払い額（PMT, Payment）
    let i_yr = iyr / 100.0 / 12.0;
    let n = ((fv * i_yr + pmt).ln() - (pv * i_yr + pmt).ln()) / (1.0 + i_yr).ln();
    -n
}

#[allow(dead_code)]
fn fv_loop(n: f64, iyr: f64, pv: f64, pmt: f64) -> f64 {
    // 比較テスト用の関数
    // 愚直にループで計算
    let i_yr = iyr / 100.0 / 12.0;
    let mut fv = pv;
    for _ in 0..(n as i32) {
        fv = fv * (1.0 + i_yr) + pmt;
    }
    -fv
}

#[test]
fn finance_test() {
    // println!("{}", number_of_periods(3.0, -20.0, -2.0, 0.0));
    assert!((fv_loop(12.0, 20.0, -22.0, -1.0) - 39.99).abs() < 0.01);
    assert!((fv_loop(30.0, 10.0, 100.0, -2.0) - -60.4226).abs() < 0.01);
    assert!((future_value(12.0, 20.0, -22.0, -1.0) - 39.99).abs() < 0.01);
    assert!((payment(12.0, 20.0, -22.0, -40.0) - 1.0).abs() < 0.01);
    assert!((future_value(30.0, 10.0, 100.0, -2.0) - -60.4226).abs() < 0.01);
}
