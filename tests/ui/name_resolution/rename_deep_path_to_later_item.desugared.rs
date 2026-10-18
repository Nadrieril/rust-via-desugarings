fn main() -> () {
    fn main__outer() -> () {
        fn main__outer__inner() -> () {
            main__later();
        }
        main__outer__inner();
    }
    main__outer();
    fn main__later() -> () {}
}
