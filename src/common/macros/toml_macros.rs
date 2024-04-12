macro_rules! change_or_insert {
    ($table:expr, $key:expr, $value:expr) => {
        if $table.get($key).is_none() {
            $table.insert($key.to_string(), $value);
        } else {
            $table[$key] = $value;
        }
    };
}

pub(crate) use change_or_insert;