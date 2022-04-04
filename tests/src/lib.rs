
#[cfg(test)]
mod tests {
    use tree_display::TreeDisplay;
    use diff_assert::try_diff;
    use tree_display_macros::TreeDisplay;

    #[derive(TreeDisplay)]
    enum TestEnum1 {
        First(usize),
        Second(TestStruct2),
        Third { seventh: usize, eigthth: usize, derp: usize },
    }

    #[derive(TreeDisplay)]
    struct TestStruct5;

    #[derive(TreeDisplay)]
    struct TestStruct4<'a, T>(&'a usize, String, T)
    where
        T: TreeDisplay;

    #[derive(TreeDisplay)]
    struct TestStruct3 {
        pub fifth: usize,
        pub sixth: usize,
    }

    #[derive(TreeDisplay)]
    struct TestStruct2 {
        pub third: usize,
        pub fourth: TestStruct3,
    }

    #[derive(TreeDisplay)]
    struct TestStruct1<'a, T>
    where
        T: TreeDisplay,
    {
        pub first: TestStruct2,
        pub second: TestStruct3,
        pub tenth: TestStruct4<'a, T>,
        pub eleventh: TestStruct5,
        pub derp: &'a TestStruct5,
        pub t: Box<T>,
        pub nineth: TestEnum1,
    }

    impl<'a, T: TreeDisplay> std::fmt::Display for TestStruct1<'a, T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.tree_fmt(f, "", false)
        }
    }

    #[test]
    fn all_inclusive() {
        let expected_file = "../tests/data/all_inclusive/test.txt";

        let derp = TestStruct5;
        let data = TestStruct1 {
            first: TestStruct2 {
                third: 1,
                fourth: TestStruct3 { fifth: 2, sixth: 3 },
            },
            second: TestStruct3 { fifth: 4, sixth: 5 },
            tenth: TestStruct4(&6, "7".to_string(), true),
            eleventh: TestStruct5,
            derp: &derp,
            t: Box::new(true),
            nineth: TestEnum1::Third {
                seventh: 8,
                eigthth: 9,
                derp: 10,
            },
        };
        let actual = data.to_string();
        let expected = match std::fs::read_to_string(expected_file) {
            Ok(s) => s.replace('\r', ""),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("Expected output file {expected_file} not found");
                    let diff_file = format!("{expected_file}.actual");
                    eprintln!("Didn't get expected output, writing to {diff_file}",);
                    std::fs::write(diff_file, actual).expect("Unable to write file");
                    panic!();
                } else {
                    panic!(
                        "Error reading expected output file {}: {}",
                        expected_file, e
                    );
                }
            }
        };
        eprintln!("{}", data);

        if let Err(e) = try_diff!(
            expected,
            actual
        ) {
            let diff_file = format!("{expected_file}.actual");
            eprintln!("Didn't get expected output, writing to {diff_file}");
            std::fs::write(diff_file, actual).expect("Unable to write file");
            panic!("{}", e);
        }
    }
}
