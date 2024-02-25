use super::super::socks5::Socks5;
use bytes::{BufMut, BytesMut};
use log;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
pub struct Service {
    host: String,
    port: i32,
    is_socks5: bool,
}

impl Service {
    pub fn new(host: String, port: i32) -> Self {
        Self {
            host,
            port,
            is_socks5: false,
        }
    }
    pub async fn run(&mut self) {
        let addr = format!("{}:{}", self.host, self.port);
        let lister = TcpListener::bind(addr).await.unwrap();
        loop {
            let (socket, _) = lister.accept().await.unwrap();
            self.process(socket).await;
        }
    }
    async fn process(&mut self, mut socket: TcpStream) {
        let mut request = webparse::Request::new();
        let mut buf = BytesMut::with_capacity(1000);
        socket.read_buf(&mut buf).await.unwrap();
        println!("{:?}", buf);
        // 判断是否是socks5
        let mut status = 0;
        let mut s = Socks5::default();
        loop {
            // 协商阶段 1
            if buf[0] == 0x05 && status == 0 {
                println!("协商阶段");
                // 返回socks5 响应
                buf.clear();
                // 服务端发送响应， 不要求用户认证
                let response: [u8; 2] = [0x05, 0x00];
                buf.put_slice(&response);
                socket.write_all(&buf).await.unwrap();
                status = 1;
                buf.clear();
                // 读取客户单发送请求
                socket.read_buf(&mut buf).await.unwrap();
            }
            // 进入第二阶段， 客户端发送请求
            else if buf[0] == 0x05 && status == 1 {
                println!("请求阶段");
                s.from(&buf);
                println!("读取{:?}", s);
                // 服务端返回请求响应
                let data = [0x05, 0x00, 0x00, s.atyp, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                let _ = socket.write_all(&data).await;
                status = 2;
            } else {
                println!("数据传输阶段");
                buf.clear();
                socket.read_buf(&mut buf).await.unwrap();
                println!("{:?}", buf);
                break;
            }
        }
        println!("开始处理请求内容");
        let _ = request.parse(&buf);
        let url = request.get_connect_url().unwrap();
        println!("{}", url);
        // let addr = url.to_socket_addrs().unwrap();
        let mut stream = TcpStream::connect(url).await.unwrap();
        println!("{}", buf.len());
        stream.write_all(&buf).await.unwrap();
        stream.flush().await.unwrap();
        buf.clear();
        stream.read_buf(&mut buf).await.unwrap();
        socket.write_all(&buf).await.unwrap();
    }
}
