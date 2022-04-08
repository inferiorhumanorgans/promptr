use crate::segment::{path::Path, ToSegment};
use crate::test::segment::declare_segement_test;
use crate::test::AppEnv;
use crate::{ApplicationState, Theme};

declare_segement_test!([
    ("PWD", "/tmp/foo"), ("HOME", "/home/username"),
]);

segment_test! {
    fn no_env() {
        |args, mut state : ApplicationState| {
            state.env.remove("PWD");
            state.env.remove("HOME");
            let seg = Path::to_segment_generic(args, &state);
            assert!(seg.is_err());
        }
    }
}

segment_test! {
    fn not_in_home_dir() {
        |args, state| {
            let seg = Path::to_segment_generic(args, &state).unwrap();
            assert_eq!(2, seg.len());
            assert_eq!("tmp", seg[0].text);
            assert_eq!("foo", seg[1].text);
        }
    }
}

segment_test! {
    fn home_dir() {
        |args, mut state : ApplicationState| {
            state.env.insert(String::from("PWD"), String::from("/home/username"));
            let seg = Path::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            assert_eq!("~", seg[0].text);
        }
    }
}

segment_test! {
    fn home_dir_subdir() {
        |args, mut state : ApplicationState| {
            state.env.insert(String::from("PWD"), String::from("/home/username/something"));
            let seg = Path::to_segment_generic(args, &state).unwrap();
            assert_eq!(2, seg.len());
            assert_eq!("~", seg[0].text);
            assert_eq!("something", seg[1].text);
        }
    }
}

segment_test! {
    #[ignore = "not yet implemented"]
    fn home_dir_no_expand() {
            let args = r##"
            {

            }
        "##;

        |args, mut state : ApplicationState| {
            state.env.insert(String::from("PWD"), String::from("/home/username/something"));
            let seg = Path::to_segment_generic(args, &state).unwrap();
            assert_eq!(2, seg.len());
            assert_eq!("~", seg[0].text);
            assert_eq!("something", seg[1].text);
        }
    }
}
