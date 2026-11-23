fn main() {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    zorness_android::main();
}
