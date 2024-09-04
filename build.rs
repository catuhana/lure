fn main() {
    if !cfg!(any(
        feature = "services-lastfm",
        feature = "services-listenbrainz"
    )) {
        println!("cargo:warning=No services are enabled. Having no features enabled removes most functionality.");
    }
}
