use game2d::geom::*;

#[test]
fn default_point_is_zero() {
    assert_eq!(P2::zero(), P2::new(0., 0.));
    assert_eq!(P2::default(), P2::zero());
}

#[test]
fn point_is_zero_check() {
    assert_eq!(P2::zero().is_zero(), true);
    assert_eq!(P2::new(0., 0.).is_zero(), true);
    assert_eq!(P2::default().is_zero(), true);
    assert_eq!(P2::new(1., 1.).is_zero(), false);
}

#[test]
fn point_x_y_accessors() {
    let pt = P2::new(1., 2.);
    assert_eq!(pt.x, 1.);
    assert_eq!(pt.y, 2.);
}

#[test]
fn point_can_be_copied() {
    fn consume_pt(_pt: P2) {}

    let pt = P2::new(1., 2.);
    consume_pt(pt);

    assert_eq!(pt, P2::new(1., 2.));
}

#[test]
fn point_can_be_debug_printed() {
    let pt = P2::new(1., 2.);
    assert_eq!(format!("{:?}", pt), "P2 { x: 1.0, y: 2.0 }");
}

#[test]
fn point_can_be_converted_from_pair() {
    let pt: P2 = [1., 2.].into();

    assert_eq!(pt, P2::new(1., 2.));
}

#[test]
fn point_can_be_converted_from_vector() {
    let pt: P2 = V2::new(1., 2.).into();
    assert_eq!(pt, P2::new(1., 2.));
}

#[test]
fn point_plus_vector_equals_point() {
    let mut pt = P2::new(1., 2.);
    let vec = V2::new(3., 4.);

    assert_eq!(pt + vec, P2::new(4., 6.));

    pt += vec;
    assert_eq!(pt, P2::new(4., 6.));
}

#[test]
fn point_minus_vector_equals_point() {
    let mut pt = P2::new(4., 6.);
    let vec = V2::new(3., 4.);

    assert_eq!(pt - vec, P2::new(1., 2.));

    pt -= vec;
    assert_eq!(pt, P2::new(1., 2.));
}

#[test]
fn point_minus_point_equals_vector() {
    let pt2 = P2::new(4., 6.);
    let pt1 = P2::new(1., 2.);

    assert_eq!(pt2 - pt1, V2::new(3., 4.));
}

#[test]
fn point_can_be_multiplied_by_a_scalar() {
    let mut pt = P2::new(10., 100.);

    assert_eq!(pt * 5., P2::new(50., 500.));

    pt *= 5.;
    assert_eq!(pt, P2::new(50., 500.));
}

#[test]
fn point_can_be_multiplied_by_a_pair() {
    let mut pt = P2::new(10., 100.);

    assert_eq!(pt * [5., 2.], P2::new(50., 200.));

    pt *= [5., 2.];
    assert_eq!(pt, P2::new(50., 200.));
}

#[test]
fn point_can_be_divided_by_a_scalar() {
    let mut pt = P2::new(50., 500.);

    assert_eq!(pt / 5., P2::new(10., 100.));

    pt /= 5.;
    assert_eq!(pt, P2::new(10., 100.));
}

#[test]
fn point_can_be_divided_by_a_pair() {
    let mut pt = P2::new(50., 200.);

    assert_eq!(pt / [5., 2.], P2::new(10., 100.));

    pt /= [5., 2.];
    assert_eq!(pt, P2::new(10., 100.));
}

#[test]
fn default_vector_is_zero() {
    assert_eq!(V2::zero(), V2::new(0., 0.));
    assert_eq!(V2::default(), V2::zero());
}

#[test]
fn vector_is_zero_check() {
    assert_eq!(V2::zero().is_zero(), true);
    assert_eq!(V2::new(0., 0.).is_zero(), true);
    assert_eq!(V2::default().is_zero(), true);
    assert_eq!(V2::new(1., 1.).is_zero(), false);
}

#[test]
fn vector_x_y_accessors() {
    let vec = V2::new(1., 2.);
    assert_eq!(vec.x, 1.);
    assert_eq!(vec.y, 2.);
}

#[test]
fn vector_can_be_copied() {
    fn consume_vec(_vec: V2) {}

    let vec = V2::new(1., 2.);
    consume_vec(vec);

    assert_eq!(vec, V2::new(1., 2.));
}

#[test]
fn vector_can_be_debug_printed() {
    let vec = V2::new(1., 2.);
    assert_eq!(format!("{:?}", vec), "V2 { x: 1.0, y: 2.0 }");
}

#[test]
fn vector_can_be_converted_from_pair() {
    let vec: V2 = [1., 2.].into();

    assert_eq!(vec, V2::new(1., 2.));
}

#[test]
fn vector_can_be_converted_from_point() {
    let vec: V2 = P2::new(1., 2.).into();
    assert_eq!(vec, V2::new(1., 2.));
}

#[test]
fn vector_plus_vector_equals_vector() {
    let mut vec1 = V2::new(1., 2.);
    let vec2 = V2::new(3., 4.);

    assert_eq!(vec1 + vec2, V2::new(4., 6.));

    vec1 += vec2;
    assert_eq!(vec1, V2::new(4., 6.));
}

#[test]
fn vector_minus_vector_equals_vector() {
    let mut vec1 = V2::new(4., 6.);
    let vec2 = V2::new(3., 4.);

    assert_eq!(vec1 - vec2, V2::new(1., 2.));

    vec1 -= vec2;
    assert_eq!(vec1, V2::new(1., 2.));
}

#[test]
fn vector_can_be_multiplied_by_a_scalar() {
    let mut vec = V2::new(10., 100.);

    assert_eq!(vec * 5., V2::new(50., 500.));

    vec *= 5.;
    assert_eq!(vec, V2::new(50., 500.));
}

#[test]
fn vector_can_be_multiplied_by_a_pair() {
    let mut vec = V2::new(10., 100.);

    assert_eq!(vec * [5., 2.], V2::new(50., 200.));

    vec *= [5., 2.];
    assert_eq!(vec, V2::new(50., 200.));
}

#[test]
fn vector_can_be_divided_by_a_scalar() {
    let mut vec = V2::new(50., 500.);

    assert_eq!(vec / 5., V2::new(10., 100.));

    vec /= 5.;
    assert_eq!(vec, V2::new(10., 100.));
}

#[test]
fn vector_can_be_divided_by_a_pair() {
    let mut vec = V2::new(50., 200.);

    assert_eq!(vec / [5., 2.], V2::new(10., 100.));

    vec /= [5., 2.];
    assert_eq!(vec, V2::new(10., 100.));
}

#[test]
fn vector_len_and_normalized() {
    let mut vec = V2::new(6., 8.);

    assert_eq!(vec.len2(), 100.);
    assert_eq!(vec.len(), 10.);
    assert_eq!(vec.normalized(), V2::new(0.6, 0.8));
    assert_eq!(vec.normalized().len(), 1.);

    vec.normalize();
    assert_eq!(vec, V2::new(0.6, 0.8));
    assert_eq!(vec.len(), 1.);

    assert_eq!(V2::zero().normalized(), V2::zero());
}
