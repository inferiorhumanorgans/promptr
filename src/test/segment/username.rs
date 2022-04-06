use crate::segment::{username::Username, ToSegment};
use crate::test::segment::declare_segement_test;
use crate::test::AppEnv;
use crate::{ApplicationState, Theme};

declare_segement_test!([
    ("USER", "newbie"),
]);

segment_test! {
    fn expected_environment() {
        |args, state| {
            let seg = Username::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!("newbie", seg.text);
        }
    }
}

segment_test! {
    fn env_has_no_user_var() {
        |args, mut state : ApplicationState| {
            state.env.remove("USER");
            let seg = Username::to_segment_generic(args, &state);

            assert!(seg.is_err());
        }
    }
}
