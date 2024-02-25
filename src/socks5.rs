use std::net::Ipv6Addr;

#[derive(Debug, Default)]
pub struct Socks5 {
    ver: u8,
    pub cmd: u8,
    rsv: u8,
    pub atyp: u8,
    pub addr: String,
    addr_len: u8,
    pub port: u16,
    pub rep: u8,
}

impl Socks5 {
    /// 支持版本5
    pub fn from(&mut self, data: &[u8]) {
        // 版本
        let ver = data[0];
        if ver != 0x05 {
            panic!("不知")
        }
        let cmd = data[1];
        let rsv = data[2];
        let atyp = data[3];
        let mut addr = "".to_string();
        let mut addr_len: u8 = 0;
        let index = 4;
        match atyp {
            // IPV4
            0x01 => {
                addr = format!("{}:{}:{}:{}", data[4], data[5], data[6], data[7]);
                addr_len = 4;
            }
            // 域名
            0x03 => {
                let len = data[4] as usize;
                addr = std::str::from_utf8(&data[5..5 + len]).unwrap().to_string();
                addr_len = len as u8;
            }
            0x04 => {
                let mut ip: [u8; 16] = [0; 16];
                for i in 0..16 {
                    ip[i as usize] = data[4 + i as usize];
                }
                addr = Ipv6Addr::from(ip).to_string();
                addr_len = 16;
            }
            _ => {}
        }
        let p: u16 = (data[index + addr_len as usize] as u16) << 8;
        let port: u16 = p + data[index + addr_len as usize + 1] as u16;
        self.ver = ver;
        self.cmd = cmd;
        self.rsv = rsv;
        self.atyp = atyp;
        self.addr = addr;
        self.addr_len = addr_len;
        self.port = port;
        self.rep = 0x00;
    }
    pub fn as_bytes(self) -> Vec<u8> {
        let mut data = Vec::<u8>::new();
        data.push(self.ver);
        data.push(self.rep);
        data.push(self.atyp);
        data.push(0);
        data.push(0);
        data
    }
}
