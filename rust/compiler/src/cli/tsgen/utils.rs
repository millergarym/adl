pub fn rel_import(src: &String, dst: &String) -> String {
    let src_v: Vec<&str> = src.split(['.']).collect();
    let src_v = &src_v[..src_v.len() - 1];
    let dst_v: Vec<&str> = dst.split(['.']).collect();
    let last = dst_v.last().unwrap();
    let dst_v = &dst_v[..dst_v.len() - 1];
    let mut src_i = src_v.iter().peekable();
    let mut dst_i = dst_v.iter().peekable();
    let mut import = String::new();
    import.push_str(".");
    while let (Some(sel), Some(del)) = (&src_i.peek(), &dst_i.peek()) {
        if sel != del {
            break;
        }
        src_i.next();
        dst_i.next();
    }
    while let Some(_) = &src_i.next() {
        import.push_str("/..");
    }
    while let Some(del) = &dst_i.next() {
        import.push_str("/");
        import.push_str(del);
    }
    import.push_str("/");
    import.push_str(last);
    import
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_import() {
        let tests: Vec<(&str, &str, &str, &str)> = vec![
            ("test 00", "abc", "runtime.adl", "./runtime/adl"),
            ("test 01", "scopedname.def", "scopedname.abc", "./abc"),
            (
                "test 02",
                "scopedname.def",
                "scopedname.def.abc",
                "./def/abc",
            ),
            (
                "test 03",
                "scopedname.def",
                "runtime.adl",
                "./../runtime/adl",
            ),
            ("test 04", "common.adminui.api", "common", "./../../common"),
            ("test 05", "common.adminui.api", "common.db", "./../db"),
            ("test 06", "common.adminui.api", "common.adminui", "./../adminui"),
        ];

        for t in tests {
            assert_eq!(
                rel_import(&t.1.to_string(), &t.2.to_string()),
                t.3,
                "{}",
                t.0
            );
        }
    }
}
