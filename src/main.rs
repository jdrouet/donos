use tokio::net::UdpSocket;

mod buffer;
mod packet;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:4545").await?;
    let mut buffer = [0; 512];
    loop {
        let (len, addr) = sock.recv_from(&mut buffer).await?;
        println!("{:?} bytes received from {:?}", len, addr);

        let len = sock.send_to(&buffer[..len], addr).await?;
        println!("{:?} bytes sent", len);
    }
}
