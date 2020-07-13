// Define and export a specific port module (transparently pull
// its namespace to the current one)
#[macro_export]
macro_rules! port {
    ($mod:ident) => {
        pub mod $mod;
        pub use self::$mod::*;
    };
    ($mod:ident as $name:ident) => {
        pub mod $mod;
        pub use self::$mod as $name;
    };
    ($outer:ident::$inner:ident) => {
        pub mod $outer { pub mod $inner; }
        pub use self::$outer::$inner::*;
    };
    ($outer:ident::$inner:ident as $name:ident) => {
        pub mod $outer { pub mod $inner; }
        pub use self::$outer::$inner as $name;
    };
}
