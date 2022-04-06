use crate::segment::{screen::Screen, ToSegment};
use crate::test::segment::declare_segement_test;
use crate::test::AppEnv;
use crate::{ApplicationState, Theme};

declare_segement_test!([
    ("STY", "1234.ttyNN.hostname"),
    ("WINDOW", "1"),
]);

segment_test! {
    fn no_screen() {
        |args, mut state : ApplicationState| {
            state.env.remove("STY");
            let seg = Screen::to_segment_generic(args, &state).unwrap();
            assert_eq!(0, seg.len());
        }
    }
}

segment_test! {
    fn screen_no_window() {
        |args, mut state : ApplicationState| {
            state.env.remove("WINDOW");

            let seg = Screen::to_segment_generic(args, &state).unwrap();
            assert_eq!(0, seg.len());
        }
    }
}

segment_test! {
    fn screen() {
        |args, state : ApplicationState| {
            let seg = Screen::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];
            assert_eq!("1[ttyNN.hostname] \u{1f4fa}", seg.text);
        }
    }
}

segment_test! {
    fn screen_no_icon() {
        let args = r##"
            {
                "show_screen_icon": false
            }
        "##;

        |args, state : ApplicationState| {
            let seg = Screen::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];
            assert_eq!("1[ttyNN.hostname]", seg.text);
        }
    }
}

segment_test! {
    fn screen_just_the_number() {
        let args = r##"
            {
                "show_screen_icon": false,
                "show_screen_name": false,
                "show_screen_pid": false
            }
        "##;

        |args, state : ApplicationState| {
            let seg = Screen::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];
            assert_eq!("1", seg.text);
        }
    }
}
