// 時間価値計算(TVM)
// 符号規約: 受け取り(+) / 支払い(-) として、以下の関係式を満たす (期末払い・月複利)
//   pv + pmt * (1 - v) / i + fv * v = 0   (v = (1 + i)^-n, i = 月利)
// 利率 0 の場合は  pv + pmt * n + fv = 0

fn monthly_rate(iyr: f64) -> f64 {
    // 年利(%) -> 月利
    iyr / 100.0 / 12.0
}

pub fn future_value(n: f64, iyr: f64, pv: f64, pmt: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 各期の支払い額（PMT, Payment）、支払い回数（N, Number of Periods）
    let i = monthly_rate(iyr);
    if i == 0.0 {
        return -(pv + pmt * n);
    }
    let growth = (1.0 + i).powf(n);
    -(pv * growth + pmt * (growth - 1.0) / i)
}

pub fn payment(n: f64, iyr: f64, pv: f64, fv: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 未来価値（FV, Future Value）、支払い回数（N, Number of Periods）
    let i = monthly_rate(iyr);
    if i == 0.0 {
        return -(pv + fv) / n;
    }
    let growth = (1.0 + i).powf(n);
    -(fv + pv * growth) * i / (growth - 1.0)
}

pub fn present_value(n: f64, iyr: f64, pmt: f64, fv: f64) -> f64 {
    // 未来価値（FV, Future Value）、利率（IYR, Interest Rate per Year）
    // 各期の支払い額（PMT, Payment）、支払い回数（N, Number of Periods）
    let i = monthly_rate(iyr);
    if i == 0.0 {
        return -(fv + pmt * n);
    }
    let discount = (1.0 + i).powf(-n);
    -(fv * discount + pmt * (1.0 - discount) / i)
}

pub fn i_yr(n: f64, pv: f64, pmt: f64, fv: f64) -> Option<f64> {
    // 年換算利率(I%YR)を二分法で求める。
    // 探索範囲 -99% 〜 1000% で符号変化がなければ解なしとして None を返す。
    let f = |iyr| present_value(n, iyr, pmt, fv) - pv;
    let (mut low, mut high) = (-99.0, 1000.0);
    let (mut f_low, f_high) = (f(low), f(high));

    if !f_low.is_finite() || !f_high.is_finite() {
        return None;
    }
    if f_low == 0.0 {
        return Some(low);
    }
    if f_high == 0.0 {
        return Some(high);
    }
    if f_low.signum() == f_high.signum() {
        return None;
    }

    for _ in 0..200 {
        let mid = (low + high) / 2.0;
        let f_mid = f(mid);
        if f_mid == 0.0 || (high - low) < 1e-12 {
            return Some(mid);
        }
        if f_mid.signum() == f_low.signum() {
            low = mid;
            f_low = f_mid;
        } else {
            high = mid;
        }
    }
    Some((low + high) / 2.0)
}

pub fn number_of_periods(iyr: f64, pv: f64, pmt: f64, fv: f64) -> f64 {
    // 現在価値（PV, Present Value）、利率（IYR, Interest Rate per Year）
    // 未来価値（FV, Future Value）、各期の支払い額（PMT, Payment）
    // 解が存在しない場合は NaN / inf を返す
    let i = monthly_rate(iyr);
    if i == 0.0 {
        return -(pv + fv) / pmt;
    }
    let annuity = pmt / i;
    -(-(pv + annuity) / (fv - annuity)).ln() / (1.0 + i).ln()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn finance_test() {
        assert!(approx(fv_loop(12.0, 20.0, -22.0, -1.0), 39.99));
        assert!(approx(fv_loop(30.0, 10.0, 100.0, -2.0), -60.4226));
        assert!(approx(
            fv_loop(30.0, 10.0, 100.0, -2.0),
            future_value(30.0, 10.0, 100.0, -2.0)
        ));
        assert!(approx(future_value(12.0, 20.0, -22.0, -1.0), 39.99));
        assert!(approx(payment(12.0, 20.0, 22.0, 0.0), -2.038));
        assert!(approx(future_value(30.0, 10.0, 100.0, -2.0), -60.4226));
        assert!(approx(present_value(12.0, 10.0, -1.0, 22.0), -8.54017));
        assert!(approx(present_value(24.0, 10.0, -1.0, 60.0), -27.49372));
        assert!(approx(present_value(6.0, 33.517, -5.5, 0.0), 30.0));
        assert!(approx(number_of_periods(10.0, 50.0, -1.0, 0.0), 64.94871));
        assert!(approx(number_of_periods(10.0, 20.0, 0.0, -25.0), 26.88864));
        assert!(approx(i_yr(6.0, -20.0, -1.0, 30.0).unwrap(), 33.287));
        assert!(approx(i_yr(6.0, -20.0, 0.0, 30.0).unwrap(), 83.896));
        assert!(approx(i_yr(6.0, 30.0, -5.5, 0.0).unwrap(), 33.517));
    }

    #[test]
    fn payment_with_future_value() {
        // 毎月 1 ずつ積み立てて fv を得る -> pmt は -1 に戻るはず
        let fv = fv_loop(12.0, 20.0, 0.0, -1.0);
        assert!(approx(payment(12.0, 20.0, 0.0, fv), -1.0));
    }

    #[test]
    fn number_of_periods_general_case() {
        // pmt, fv が共に 0 でないケース (以前は powf の誤りで NaN になっていた)
        let fv = future_value(24.0, 10.0, 50.0, -3.0);
        assert!(approx(number_of_periods(10.0, 50.0, -3.0, fv), 24.0));
    }

    #[test]
    fn zero_interest_rate() {
        assert!(approx(future_value(10.0, 0.0, 100.0, -5.0), -50.0));
        assert!(approx(payment(10.0, 0.0, 100.0, -50.0), -5.0));
        assert!(approx(present_value(10.0, 0.0, -5.0, -50.0), 100.0));
        assert!(approx(number_of_periods(0.0, 100.0, -5.0, -50.0), 10.0));
    }

    #[test]
    fn i_yr_round_trip() {
        for &rate in &[-5.0, 0.5, 3.0, 12.0, 150.0] {
            let fv = future_value(36.0, rate, 100.0, -2.0);
            assert!(approx(i_yr(36.0, 100.0, -2.0, fv).unwrap(), rate));
        }
    }

    #[test]
    fn i_yr_no_solution() {
        // pv と fv が同符号で支払いもない -> 解なし
        assert!(i_yr(12.0, 100.0, 0.0, 100.0).is_none());
    }
}
