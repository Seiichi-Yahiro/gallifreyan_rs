macro_rules! update_if_changed {
    ($old:expr, $new:expr, $text:expr) => {
        if $old != $new {
            debug!($text, $old, $new);
            $old = $new;
        }
    };
}

pub(crate) use update_if_changed;
