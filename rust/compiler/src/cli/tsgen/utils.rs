pub fn rel_import(src: &String, dst: &String) -> String {
    let src_v: Vec<&str> = src.split(['.']).collect();
    // strip off the name, only leave the path
    let src_v = &src_v[..src_v.len() - 1];
    let dst_v: Vec<&str> = dst.split(['.']).collect();
    let mut common = 0;
    let mut src_i = src_v.iter();
    let mut dst_i = dst_v.iter();
    let mut import = String::new();
    import.push_str("./");
    while let (Some(sel), Some(del)) = (&src_i.next(), &dst_i.next()) {
        if sel != del {
            break;
        }
        common += 1;
    }
    // dbg!(&src_v);
    // dbg!(common);
    // dbg!(&dst_v);
    import.push_str("../".repeat(src_v.len() - common).as_str());
    if common == dst_v.len() {
        import.push_str("../");
        import.push_str(dst_v.join("/").as_str());
    } else {
        import.push_str(dst_v[common..].join("/").as_str());
    }
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
            ("test 06", "common.adminui.api", "common.adminui", "./../common/adminui"),
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
