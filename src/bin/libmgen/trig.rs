use crate::helpers;
use crate::test::gen_test;
use doctor_syn::num_digits_for;
use doctor_syn::Parity;
use doctor_syn::{expr, name};
use proc_macro2::TokenStream;
use quote::quote;
use std::f64::consts::PI;

pub fn gen_quadrant_sin(num_terms: usize, num_bits: usize, number_type: &str) -> TokenStream {
    let fty = helpers::get_fty(num_bits);

    // Quadrant sin/cos over a smaller range.
    let xmin = -0.25;
    let xmax = 0.25;

    let sin_approx = expr!((s * PI).sin())
        .approx(
            num_terms,
            xmin,
            xmax,
            name!(s),
            Parity::Odd,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    let cos_approx = expr!(-(c * PI).cos())
        .approx(
            num_terms + 1,
            xmin,
            xmax,
            name!(c),
            Parity::Even,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    quote!(
        fn sin(arg: #fty) -> #fty {
            let scaled = arg * (1.0 / PI);
            let xh = x + 0.5;
            let xr = x.round();
            let xhr = xh.round();
            let s = x - xr;
            let c = xh - xhr;
            let sr = #sin_approx;
            let cr = #cos_approx;
            let ss = negate_on_odd(xr, sr);
            let cs = negate_on_odd(xhr, cr);
            let ss2 = if (xr as i32) & 1 == 0 { sr } else { -sr };
            let cs2 = if (xhr as i32 & 1) == 0 { cr } else { -cr };
            assert_eq!(ss, ss2);
            assert_eq!(cs, cs2);
            if s.abs() <= 0.25 { ss } else { cs }
        }
    )
}

pub fn gen_quadrant_cos(num_terms: usize, num_bits: usize, number_type: &str) -> TokenStream {
    let fty = helpers::get_fty(num_bits);

    // Quadrant sin/cos over a smaller range.
    let xmin = -0.25;
    let xmax = 0.25;

    let sin_approx = expr!((s * PI).sin())
        .approx(
            num_terms,
            xmin,
            xmax,
            name!(s),
            Parity::Odd,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    let cos_approx = expr!((c * PI).cos())
        .approx(
            num_terms + 1,
            xmin,
            xmax,
            name!(c),
            Parity::Even,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    quote!(
        fn cos(arg: #fty) -> #fty {
            let x = arg * (1.0 / PI);
            let xh = x + 0.5;
            let xr = x.round();
            let xhr = xh.round();
            let c = x - xr;
            let s = xh - xhr;
            let sr = #sin_approx;
            let cr = #cos_approx;
            let ss = if xhr as i32 & 1 == 0 { sr } else { -sr };
            let cs = if xr as i32 & 1 == 0 { cr } else { -cr };
            if s.abs() <= 0.25 { ss } else { cs }
        }
    )
}

#[allow(dead_code)]
pub fn gen_single_pass_sin(num_terms: usize, num_bits: usize, number_type: &str) -> TokenStream {
    let fty = helpers::get_fty(num_bits);

    let xmin = -0.5;
    let xmax = 0.5;

    let approx = expr!((x * PI * 2.0).sin())
        .approx(
            num_terms,
            xmin,
            xmax,
            name!(x),
            Parity::Odd,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    quote!(
        fn sin(arg: #fty) -> #fty {
            let scaled : #fty = arg * (1.0 / (PI * 2.0));
            let x : #fty = scaled - scaled.round();
            #approx
        }
    )
}

#[allow(dead_code)]
pub fn gen_single_pass_cos(num_terms: usize, num_bits: usize, number_type: &str) -> TokenStream {
    let fty = helpers::get_fty(num_bits);

    let xmin = -0.5;
    let xmax = 0.5;

    let approx = expr!((x * PI * 2.0).cos())
        .approx(
            num_terms,
            xmin,
            xmax,
            name!(x),
            Parity::Even,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    quote!(
        fn cos(arg: #fty) -> #fty {
            let scaled : #fty = arg * (1.0 / (PI * 2.0));
            let x : #fty = scaled - scaled.round();
            #approx
        }
    )
}

pub fn gen_sin_cos(_num_terms: usize, num_bits: usize, _number_type: &str) -> TokenStream {
    let fty = helpers::get_fty(num_bits);

    // There is some synergy between sin and cos, but not as much as ULP-focused approximants.
    quote!(
        fn sin_cos(arg: #fty) -> (#fty, #fty) {
            (sin(arg), cos(arg))
        }
    )
}

pub fn gen_tan(num_terms: usize, num_bits: usize, number_type: &str) -> TokenStream {
    let fty = helpers::get_fty(num_bits);

    // Use a Padé approximation. The expression (x*x - pi*pi/4) goes to zero at the poles
    // cancelling the infinities, similar to sinc(x).
    let xmin = -0.499999;
    let xmax = 0.499999;

    let approx = expr!((x * PI).tan() * (x * x - 0.25))
        .approx(
            num_terms,
            xmin,
            xmax,
            name!(x),
            Parity::Odd,
            num_digits_for(num_bits),
        )
        .unwrap()
        .use_number_type(number_type)
        .unwrap()
        .into_inner();

    // TODO: calculate the recipocal without a divide.
    quote!(
        fn tan(arg: #fty) -> #fty {
            let scaled : #fty = arg * (1.0 / PI);
            let x : #fty = scaled - scaled.round();
            let recip : #fty = 1.0 / (x*x - 0.25);
            let y : #fty = #approx ;
            y * recip
        }
    )
}

// Generate accurate sin, cos, tan, sin_cos.
// Return functions and tests.
#[allow(dead_code)]
pub fn gen_quadrant_trig(num_bits: usize, number_type: &str) -> (TokenStream, TokenStream) {
    let cos_sin_num_terms = helpers::get_quadrant_terms(num_bits);
    let tan_num_terms = helpers::get_tan_terms(num_bits);
    let sin = gen_quadrant_sin(cos_sin_num_terms, num_bits, number_type);
    let cos = gen_quadrant_cos(cos_sin_num_terms, num_bits, number_type);
    let tan = gen_tan(tan_num_terms, num_bits, number_type);
    let sin_cos = gen_sin_cos(cos_sin_num_terms, num_bits, number_type);

    let fty = helpers::get_fty(num_bits);

    let bit = (2.0_f64).powi(if num_bits == 32 { -23 } else { -52 });

    let test_sin = gen_test(
        quote!(test_sin),
        quote!(x.sin()),
        quote!(sin(x as #fty) as f64),
        bit * 3.0,
        -PI,
        PI,
    );
    let test_cos = gen_test(
        quote!(test_cos),
        quote!(x.cos()),
        quote!(cos(x as #fty) as f64),
        bit * 4.0,
        -PI,
        PI,
    );
    let test_tan = gen_test(
        quote!(test_tan),
        quote!(x.tan()),
        quote!(tan(x as #fty) as f64),
        bit * 6.0,
        -PI / 4.0,
        PI / 4.0,
    );
    let test_sin_cos_1 = gen_test(
        quote!(test_sin_cos_1),
        quote!(x.sin()),
        quote!(sin_cos(x as #fty).0 as f64),
        bit * 3.0,
        -PI,
        PI,
    );
    let test_sin_cos_2 = gen_test(
        quote!(test_sin_cos_2),
        quote!(x.cos()),
        quote!(sin_cos(x as #fty).1 as f64),
        bit * 4.0,
        -PI,
        PI,
    );

    (
        quote! {
            #sin #cos #tan #sin_cos
        },
        quote! {
            #test_sin #test_cos #test_tan #test_sin_cos_1 #test_sin_cos_2
        },
    )
}

pub fn gen_single_pass_trig(num_bits: usize, number_type: &str) -> (TokenStream, TokenStream) {
    // let cos_sin_num_terms = helpers::get_quadrant_terms(num_bits);
    // let tan_num_terms = helpers::get_tan_terms(num_bits);
    // let sin = gen_quadrant_sin(cos_sin_num_terms, num_bits, number_type);
    // let cos = gen_quadrant_cos(cos_sin_num_terms, num_bits, number_type);

    let cos_sin_num_terms = helpers::get_single_pass_terms(num_bits);
    let tan_num_terms = helpers::get_tan_terms(num_bits);
    let sin = gen_single_pass_sin(cos_sin_num_terms, num_bits, number_type);
    let cos = gen_single_pass_cos(cos_sin_num_terms + 1, num_bits, number_type);

    let tan = gen_tan(tan_num_terms, num_bits, number_type);
    let sin_cos = gen_sin_cos(cos_sin_num_terms, num_bits, number_type);

    let fty = helpers::get_fty(num_bits);

    let bit = (2.0_f64).powi(if num_bits == 32 { -23 } else { -52 });

    let test_sin = gen_test(
        quote!(test_sin),
        quote!(x.sin()),
        quote!(sin(x as #fty) as f64),
        bit * 8.0,
        -PI,
        PI,
    );
    let test_cos = gen_test(
        quote!(test_cos),
        quote!(x.cos()),
        quote!(cos(x as #fty) as f64),
        bit * 8.0,
        -PI,
        PI,
    );
    let test_tan = gen_test(
        quote!(test_tan),
        quote!(x.tan()),
        quote!(tan(x as #fty) as f64),
        bit * 6.0,
        -PI / 4.0,
        PI / 4.0,
    );
    let test_sin_cos_1 = gen_test(
        quote!(test_sin_cos_1),
        quote!(x.sin()),
        quote!(sin_cos(x as #fty).0 as f64),
        bit * 8.0,
        -PI,
        PI,
    );
    let test_sin_cos_2 = gen_test(
        quote!(test_sin_cos_2),
        quote!(x.cos()),
        quote!(sin_cos(x as #fty).1 as f64),
        bit * 8.0,
        -PI,
        PI,
    );

    (
        quote! {
            #sin #cos #tan #sin_cos
        },
        quote! {
            #test_sin #test_cos #test_tan #test_sin_cos_1 #test_sin_cos_2
        },
    )
}
