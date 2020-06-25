use log::{error, info, warn};
use socket::clients::Client;
use socket::clients::Clients;
use socket::message::MsgData;
use socket::epevent::EpEvent;
use socket::epoll::Epoll;
use socket::tcp_listen::TcpListen;
use std::thread;

pub fn test() {
    client();

    match server() {
        Ok(joinhandle) => joinhandle.join().unwrap(),
        Err(err) => error!("{}", err),
    }
}

fn client() -> thread::JoinHandle<()> {
    std::thread::spawn(move || {
        info!("client-->{:?}", std::thread::current().id());
        thread::sleep(std::time::Duration::from_secs(5));

        match std::net::TcpStream::connect("0.0.0.0:9988") {
            Ok(tcp_strem) => {
                warn!("connect success:{:?}", tcp_strem.local_addr());
                let mut client = Client::new(tcp_strem, 1024, |net_data| {
                    warn!("id:{} buffer:{:?}", net_data.id, net_data.data);
                });

                thread::sleep(std::time::Duration::from_secs(1));

                let mut buffer = vec![0u8; "hello world".as_bytes().len()];
                utils::bytes::write_bytes(&mut buffer, "hello world".as_bytes());
                let netdata = Box::new(MsgData {
                    id: 1,
                    data: buffer,
                });

                client.tcp_writer.add_msgdata(netdata);

                match client.tcp_writer.write(&mut client.stream) {
                    Ok(result) => {
                        warn!("write result:{:?}", result);
                        thread::sleep(std::time::Duration::from_secs(1));
                        match client.tcp_reader.read(&mut client.stream) {
                            Ok(result) => {
                                warn!("read result:{:?}", result);
                            }
                            Err(err) => error!("write error:{}", err),
                        }
                    }
                    Err(err) => error!("write error:{}", err),
                }
            }

            Err(err) => error!("connect error:{}", err),
        }
    })
}

fn server() -> Result<thread::JoinHandle<()>, String> {
    let epoll: Epoll;
    match Epoll::new() {
        Ok(ep) => epoll = ep,
        Err(err) => {
            return Err(err);
        }
    };

    let server = thread::spawn(move || match Clients::new(99, 1024, &epoll) {
        Ok(mut clients) => {
            info!("server-->{:?}", thread::current().name());

            info!("server-->{:?}", thread::current().id());

            let mut epevent = EpEvent::new(&epoll, &mut clients);
            match TcpListen::new(&String::from("0.0.0.0:9988"), 99, &epoll, &mut epevent) {
                Ok(mut listen) => loop {
                    match listen.run(1000) {
                        Ok(()) => (),
                        Err(err) => error!("{}", err),
                    }
                },
                Err(err) => error!("tcplisten:new error:{}", err),
            }
        }
        Err(err) => error!("Clients::new error:{:?}", err),
    });

    Ok(server)
}
