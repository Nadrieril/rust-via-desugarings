fn main() -> () {
    fn main__f1() -> () {}
    fn main__f2() -> () {
        fn main__f2__f1() -> () {}
        main__f2__f1();
    }
    main__f1();
}
