use http_srvr::SRVR;

fn main() {
    let srvr = SRVR::new();
    let _ = srvr.listen("127.0.0.1", "4321");
}
