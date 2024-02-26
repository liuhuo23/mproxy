use super::super::socks5::Socks5;
use bytes::{BufMut, BytesMut};
use clap::Parser;
use log;
use mproxy::utils::{load_certs, load_keys};
use std::net::SocketAddr;
use std::sync::Arc;
use std::{io, path::PathBuf};
use tokio::{
    io::{copy, sink, split, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_rustls::TlsAcceptor;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Service {
    /// Name of the person to greet
    #[arg(short = 'H', long, default_value = "127.0.0.1", help = "输入代理地址")]
    pub host: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 8080, help = "输入代理端口")]
    pub port: i32,

    #[arg(short = 'C', long, help = "请输入cert文件路径")]
    pub cert: Option<PathBuf>,
    #[arg(short, long, help = "请输入 key 文件路径")]
    pub key: Option<PathBuf>,
}

impl Service {
    pub async fn run(&mut self) -> io::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let certs = load_certs(self.cert.as_ref())?;
        let keys = load_keys(self.key.as_ref())?;
        let lister = TcpListener::bind(addr).await?;
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, keys)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
        let acceptor = TlsAcceptor::from(Arc::new(config));
        loop {
            let (stream, peer_addr) = lister.accept().await?;
            let res = self.process(stream, acceptor.clone(), peer_addr).await;
            match res {
                Ok(()) => {}
                Err(s) => {
                    log::error!("{}", s);
                }
            }
        }
    }
    async fn process(
        &mut self,
        mut stream: TcpStream,
        acceptor: TlsAcceptor,
        peer_addr: SocketAddr,
    ) -> io::Result<()> {
        let mut request = webparse::Request::new();
        let mut buf = BytesMut::with_capacity(1000);
        stream.read_buf(&mut buf).await?;
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
                stream.write_all(&buf).await?;
                status = 1;
                buf.clear();
                // 读取客户单发送请求
                stream.read_buf(&mut buf).await?;
            }
            // 进入第二阶段， 客户端发送请求
            else if buf[0] == 0x05 && status == 1 {
                println!("请求阶段");
                s.from(&buf);
                println!("读取{:?}", s);
                // 服务端返回请求响应
                let data = [0x05, 0x00, 0x00, s.atyp, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                let _ = stream.write_all(&data).await;
                status = 2;
            } else if status == 2 {
                println!("数据传输阶段");
                buf.clear();
                stream.read_buf(&mut buf).await?;
                unsafe {
                    println!("{:?}", String::from_utf8_unchecked(buf.to_vec()));
                }
                break;
            } else {
                break;
            }
        }
        println!("开始处理请求内容");
        let res = request.parse(&buf);
        let mut https = false;
        match res {
            Ok(_) => {
                log::debug!("not https");
                https = false;
            }
            Err(_) => {
                log::debug!("is https");
                https = true;
            }
        }
        if !https {
            let url = request.get_connect_url().unwrap();
            println!("{}", url);
            // let addr = url.to_socket_addrs().unwrap();
            let mut s_stream = TcpStream::connect(url).await?;
            println!("{}", buf.len());
            s_stream.write_all(&buf).await?;
            s_stream.flush().await?;
            buf.clear();
            s_stream.read_buf(&mut buf).await?;
            stream.write_all(&buf).await?;
            Ok(())
        } else {
            log::debug!("进入ssl阶段");
            let mut stream = acceptor.accept(stream).await?;
            let flag_mode = true;
            if flag_mode {
                log::debug!("flag_mode true");
                let (mut reader, mut writer) = split(stream);
                let n = copy(&mut reader, &mut writer).await?;
                writer.flush().await?;
                println!("Echo: {} - {}", peer_addr, n);
            } else {
                log::debug!("flag_mode false");
                let mut output = sink();
                stream
                    .write_all(
                        &b"HTTP/1.0 200 ok\r\n\
                                        Connection: close\r\n\
                                        Content-length: 12\r\n\
                                        \r\n\
                                        Hello world!"[..],
                    )
                    .await?;
                stream.shutdown().await?;
                copy(&mut stream, &mut output).await?;
                println!("Hello: {}", peer_addr);
            }

            Ok(()) as io::Result<()>
        }
    }
}
