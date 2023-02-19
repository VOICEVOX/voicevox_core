use indoc::indoc;
use ndarray::Array;
use ndarray_stats::DeviationExt as _;

pub(crate) fn close_l1(test: &[f32], truth: &[f32], tol: f32) {
    // ndarray-linalgというクレートに同名の関数/マクロがあるのだが、Windowsでは
    // 謎のリンクエラーを起こす。そのため似たようなのを手作り

    let test = ndarray::arr1(test);
    let truth = &ndarray::arr1(truth);
    let dev = test.l1_dist(truth).unwrap() / test.l1_dist(&Array::zeros(test.len())).unwrap();
    if dev > tol {
        panic!(
            indoc! {"
                Too large deviation in L1-norm: {dev} > {tol}
                Expected: {truth}
                Actual:   {test}
            "},
            dev = dev,
            tol = tol,
            truth = truth,
            test = test,
        );
    }
}
