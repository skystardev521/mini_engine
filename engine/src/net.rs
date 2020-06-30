/*
use crate::config::NetConfig;
use socket::clients::Clients;
use socket::clients::NewClientsResult;
use socket::epoll::Epoll;
use socket::message::NetMsg;
use socket::tcp_event::TcpEvent;
use socket::tcp_listen::TcpListen;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

pub struct Net<'a> {
    config: NetConfig,
    epoll: Option<Epoll>,
    clients: Option<Clients<'a>>,
    tcp_event: Option<TcpEvent<'a>>,
    tcp_listen: Option<TcpListen<'a>>,
    receiver: Receiver<NetMsg>,
    sync_sender: SyncSender<NetMsg>,
}

impl<'a> Net<'a> {
    pub fn new(
        config: NetConfig,
        receiver: Receiver<NetMsg>,
        sync_sender: SyncSender<NetMsg>,
    ) -> Self {
        Net {
            config: config,
            receiver: receiver,
            sync_sender: sync_sender,
            epoll: None,
            clients: None,
            tcp_event: None,
            tcp_listen: None,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        let epoll: Epoll;
        match Epoll::new() {
            Ok(ep) => epoll = ep,
            Err(err) => return Err(err),
        };

        let mut clients: Clients;
        match Clients::new(self.config.max_client, self.config.msg_max_size, &epoll) {
            NewClientsResult::Ok(cs) => clients = cs,
            NewClientsResult::MsgSizeTooBig => {
                return Err(String::from("MsgSizeTooBig"));
            }
            NewClientsResult::ClientNumTooSmall => {
                return Err(String::from("ClientNumTooSmall"));
            }
        };

        let mut tcp_event = TcpEvent::new(&epoll, &mut clients, |msg_data| {});

        let tcp_listen: TcpListen;
        match TcpListen::new(
            &self.config.tcp_linsten_addr,
            self.config.epoll_max_events,
            &epoll,
            &mut tcp_event,
        ) {
            Ok(listen) => tcp_listen = listen,
            Err(err) => return Err(err),
        }

        /*
        match Epoll::new() {
            Ok(ep) => self.epoll = Some(ep),
            Err(err) => return Err(err),
        };

        match Clients::new(self.config.max_client, self.config.msg_max_size, epoll) {
            NewClientsResult::Ok(clients) => {
                self.clients = Some(clients);
            }
            NewClientsResult::MsgSizeTooBig => {
                return Err(String::from("MsgSizeTooBig"));
            }
            NewClientsResult::ClientNumTooSmall => {
                return Err(String::from("ClientNumTooSmall"));
            }
        };
        */

        /*
        if let (Some(epoll), Some(clients)) = (self.epoll, self.clients) {
            self.tcp_event = Some(TcpEvent::new(&epoll, &mut clients));
        }

        if let (Some(epoll), Some(tcp_event)) = (self.epoll, self.tcp_event) {
            match TcpListen::new(
                &self.config.tcp_linsten_addr,
                self.config.epoll_max_events,
                &epoll,
                &mut tcp_event,
            ) {
                Ok(listen) => self.tcp_listen = Some(listen),
                Err(err) => return Err(err),
            };
        }
        */

        /*
        match self.clients {
            Some(clients) => self.tcp_event = Some(TcpEvent::new(&epoll, &mut clients)),
            None => return Err(String::from("ClientNumTooSmall")),
        }

        let mut tcp_event = TcpEvent::new(&epoll, &mut self.clients.unwrap());

        let tcp_listen: TcpListen;
        match TcpListen::new(
            &self.config.tcp_linsten_addr,
            self.config.epoll_max_events,
            &epoll,
            &mut tcp_event,
        ) {
            Ok(listen) => tcp_listen = listen,
            Err(err) => return Err(err),
        }
        */
        Ok(())
    }
}
*/
