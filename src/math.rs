#[macro_export]
macro_rules! clamp {
    ($min:expr, $value:expr, $max:expr) => {
        max!(min!($max, $value), $min)
    };
}

#[macro_export]
macro_rules! clamp_with_bel {
    ($min:expr, $value:expr, $max: expr) => {
        {
            let old_value = $value;
            let new_value = max!(min!($max, $value), $min);
            if new_value != old_value {
                eprint!("\x07");
            }
            new_value
        }
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
