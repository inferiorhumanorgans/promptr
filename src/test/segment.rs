mod command_status;
mod hostname;
mod path;
mod screen;
mod username;

/// Expand a JSON string literal into a strongly typed object or None if we pass None.
macro_rules! test_args {
    (None) => {
        None
    };
    ($args:literal) => {
        Some(serde_json::from_str($args).expect("Invalid JSON"))
    };
}
pub(crate) use test_args;

// Fuuuck…
// https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963
macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}
pub(crate) use with_dollar_sign;

/// Meta macros at their meatiest. Use this to define macros that create a default environment
/// for test instances.  This doesn't namespace or even come up with creative names for anything
/// so this only works once per scope.
macro_rules! declare_segement_test {
    ([ $(($key:literal, $value:literal),)* ]) => {
        crate::test::segment::with_dollar_sign!{
            ($d:tt) => {
                macro_rules! segment_test {
                    ($d(#[$d meta:meta])* fn $name:ident () {
                        $body:expr
                    }) => {
                        segment_test!(
                            $d(#[$d meta])*,
                            $name,
                            None,
                            [
                                $(($key, $value),)*
                            ],
                            $body
                        );
                    };

                    ($d(#[$d meta:meta])* fn $name:ident () {
                        let args = $args:literal;

                        $body:expr
                    }) => {
                        segment_test!(
                            $d(#[$d meta])*,
                            $name,
                            $args,
                            [
                                $(($key, $value),)*
                            ],
                            $body
                        );
                    };

                    ($d(#[$d meta:meta])*, $name:ident,  $args:tt, [ $d(($d inner_key:literal, $d inner_value:literal),)* ], $body:expr) => {
                        $d(#[$d meta])*
                        #[test]
                        fn $name() {
                            let args = crate::test::segment::test_args!($args);

                            let env = AppEnv::from([
                                $d((String::from($d inner_key), String::from($d inner_value)),)*
                            ]);

                            let state = ApplicationState {
                                theme: &Theme::default(),
                                env,
                            };
                            $body(args, state);
                        }
                    };
                }
            }
        }
    };
}
pub(crate) use declare_segement_test;
