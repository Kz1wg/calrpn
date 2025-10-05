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

    (fv - pv * (1.0 + i_yr).powf(n)) / ((1.0 + i_yr).powf(n) - 1.0) * i_yr
}
pub fn present_value(n: f64, iyr: f64, pmt: f64, fv: f64) -> f64 {
    // 未来価値（FV, Future Value）、利率（IYR, Interest Rate per Year）
    // 各期の支払い額（PMT, Payment）、支払い回数（N, Number of Periods）
    let i_yr = iyr / 100.0 / 12.0;
    let pv = fv / ((1.0 + i_yr).powf(n)) + pmt * (1.0 - (1.0 + i_yr).powf(-n)) / i_yr;
    -pv
}

pub fn i_yr(n: f64, pv: f64, pmt: f64, fv: f64) -> Option<f64> {
    // 年換算利率(I%RY)
    // 二分探索法で力技で求める。
    let mut upper = 100.0;
    let mut low = 0.0;
    let mut tyr = (upper + low) / 2.0;
    let evo = |iyr| present_value(n, iyr, pmt, fv) - pv;
    for _i in 0..100 {
        tyr = (upper + low) / 2.0;
        let ee = evo(tyr);
        if ee > 0.0 {
            if pv < 0.0 {
                upper = tyr;
            } else {
                low = tyr;
            }
        } else if pv < 0.0 {
            low = tyr;
        } else {
            upper = tyr
        };
        if ee.abs() < 0.000001 {
            break;
        }
    }

    Some(tyr)
}

pub fn number_of_periods(iyr: f64, pv: f64, pmt: f64, fv: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 未来価値（FV, Future Value）、各期の支払い額（PMT, Payment）
    let i_yr = iyr / 100.0 / 12.0;
    if pmt == 0.0 {
        (fv / -pv).ln() / (1.0 + i_yr).ln()
    } else if fv == 0.0 {
        -((1.0 + (i_yr * pv) / pmt).ln() / (1.0 + i_yr).ln())
    } else {
        -(((pv + pmt.powf(1.0 + i_yr) / i_yr) / (fv - pmt.powf(1.0 + i_yr) / i_yr)).ln()
            / (1.0 + i_yr).ln())
    }
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
    assert!((fv_loop(12.0, 20.0, -22.0, -1.0) - 39.99).abs() < 0.01);
    assert!((fv_loop(30.0, 10.0, 100.0, -2.0) - -60.4226).abs() < 0.01);
    assert!(
        (fv_loop(30.0, 10.0, 100.0, -2.0) - future_value(30.0, 10.0, 100.0, -2.0)).abs() < 0.01
    );
    assert!((future_value(12.0, 20.0, -22.0, -1.0) - 39.99).abs() < 0.01);
    assert!((payment(12.0, 20.0, 22.0, 0.0) + 2.038).abs() < 0.01);
    assert!((future_value(30.0, 10.0, 100.0, -2.0) - -60.4226).abs() < 0.01);
    assert!((present_value(12.0, 10.0, -1.0, 22.0) - -8.54017).abs() < 0.01);
    assert!((present_value(24.0, 10.0, -1.0, 60.0) - -27.49372).abs() < 0.01);
    assert!((present_value(6.0, 33.517, -5.5, 0.0) - 30.0).abs() < 0.01);
    assert!((number_of_periods(10.0, 50.0, -1.0, 0.0) - 64.94871).abs() < 0.01);
    assert!((number_of_periods(10.0, 20.0, 0.0, -25.0) - 26.88864).abs() < 0.01);
    assert!((i_yr(6.0, -20.0, -1.0, 30.0).unwrap() - 33.287).abs() < 0.01);
    assert!((i_yr(6.0, -20.0, 0.0, 30.0).unwrap() - 83.896).abs() < 0.01);
    assert!((i_yr(6.0, 30.0, -5.5, 0.0).unwrap() - 33.517).abs() < 0.01);
}
