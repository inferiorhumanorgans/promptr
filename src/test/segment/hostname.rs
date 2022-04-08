use crate::segment::{hostname::Hostname, ToSegment};
use crate::test::segment::declare_segement_test;
use crate::test::AppEnv;
use crate::{ApplicationState, Theme};

declare_segement_test!([
    ("hostname", "sean.connery.is.zardoz.com"),
]);

segment_test! {
    fn hostname() {
        let args = r##"
            {
                "show_jail_indicator": false
            }
        "##;

        |args, state| {
            let seg = Hostname::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!("sean", seg.text);
        }
    }
}

segment_test! {
    fn hostname_with_domain() {
        let args = r##"
            {
                "show_domain": true,
                "show_jail_indicator": false
            }
        "##;

        |args, state| {
            let seg = Hostname::to_segment_generic(args, &state).unwrap();
            assert_eq!(1, seg.len());
            let seg = &seg[0];

            assert_eq!("sean.connery.is.zardoz.com", seg.text);
        }
    }
}
