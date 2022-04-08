use crate::segment::{command_status::CommandStatus, ToSegment};
use crate::test::segment::declare_segement_test;
use crate::test::AppEnv;
use crate::{ApplicationState, Theme};

declare_segement_test!([
]);

segment_test! {
    fn no_exit_status_non_priv() {
        |args, state| {
            let seg = CommandStatus::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!(r"\$", seg.text);
            assert_eq!(state.theme.command_status.success_bg, seg.bg);
        }
    }
}

segment_test! {
    fn exit_success_non_priv() {
        |args, mut state : ApplicationState| {
            state.env.insert(String::from("code"), String::from("0"));
            state.env.insert(String::from("uid"), String::from("1"));

            let seg = CommandStatus::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!(r"\$", seg.text);
            assert_eq!(state.theme.command_status.success_bg, seg.bg);
        }
    }
}

segment_test! {
    fn exit_fail_non_priv() {
        |args, mut state : ApplicationState| {
            state.env.insert(String::from("code"), String::from("255"));
            state.env.insert(String::from("uid"), String::from("1"));

            let seg = CommandStatus::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!(r"\$", seg.text);
            assert_eq!(state.theme.command_status.failure_bg, seg.bg);
        }
    }
}


segment_test! {
    fn exit_success_priv() {
        |args, mut state : ApplicationState| {
            state.env.insert(String::from("code"), String::from("0"));
            state.env.insert(String::from("uid"), String::from("0"));

            let seg = CommandStatus::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!(r"#", seg.text);
            assert_eq!(state.theme.command_status.success_bg, seg.bg);
        }
    }
}

segment_test! {
    fn exit_fail_priv() {
        |args, mut state : ApplicationState| {
            state.env.insert(String::from("code"), String::from("255"));
            state.env.insert(String::from("uid"), String::from("0"));

            let seg = CommandStatus::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!(r"#", seg.text);
            assert_eq!(state.theme.command_status.failure_bg, seg.bg);
        }
    }
}

