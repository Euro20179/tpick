#[macro_export]
macro_rules! clamp {
    ($min:expr, $value:expr, $max:expr) => {
        max!(min!($max, $value), $min)
    };
}

#[macro_export]
macro_rules! min {
    ($i1:expr, $i2:expr) => {
        if $i1 < $i2 {
            $i1
        } else {
            $i2
        }
    };
}

#[macro_export]
macro_rules! max {
    ($i1:expr, $i2:expr) => {
        if $i1 > $i2 {
            $i1
        } else {
            $i2
        }
    };
}
