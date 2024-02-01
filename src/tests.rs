use crate::{contrast, hsl2rgb, rgb2hsl};

#[test]
fn color_conversions() {
    assert_eq!(rgb2hsl(255.0, 0.0, 120.0), (331.7647, 100.0, 50.0));
    assert_eq!(hsl2rgb(331.7647, 100.0, 50.0), (255.0, 0.0, 120.0));
    assert_eq!(contrast([255.0, 255.0, 255.0], [0.0, 0.0, 255.0]), 8.592471);
}
