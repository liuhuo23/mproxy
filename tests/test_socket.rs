use mproxy::socks5::Socks5;

#[test]
fn add() {
    let data = b"\x05\x01\0\x01n\xf2D\x03\0P";
    let mut s = Socks5::default();
    s.from(data);
    println!("{:?}", s);
}
