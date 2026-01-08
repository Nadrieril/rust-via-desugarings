/// Returns the values of the command-line options that match `needle`. The options are built-in
/// to be of the form `--arg=value` or `--arg value`.
pub fn arg_values<'a, T: AsRef<str>>(
    args: &'a [T],
    needle: &'a str,
) -> impl Iterator<Item = &'a str> {
    struct ArgFilter<'a, T> {
        args: std::slice::Iter<'a, T>,
        needle: &'a str,
    }
    impl<'a, T: AsRef<str>> Iterator for ArgFilter<'a, T> {
        type Item = &'a str;
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(arg) = self.args.next() {
                let mut split_arg = arg.as_ref().splitn(2, '=');
                if split_arg.next() == Some(self.needle) {
                    return match split_arg.next() {
                        // `--arg=value` form
                        arg @ Some(_) => arg,
                        // `--arg value` form
                        None => self.args.next().map(|x| x.as_ref()),
                    };
                }
            }
            None
        }
    }
    ArgFilter {
        args: args.iter(),
        needle,
    }
}

pub fn arg_value<'a, T: AsRef<str>>(args: &'a [T], needle: &'a str) -> Option<&'a str> {
    arg_values(args, needle).next()
}
