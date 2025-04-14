#![allow(unused)]

use dev_utils::{
    app_dt,
    dlog::{Level, set_max_level},
};

fn main() {
    // app_dt!(file!());
    app_dt!(file!(), "package" => ["license", "keywords"]);

    set_max_level(Level::Trace);
}
