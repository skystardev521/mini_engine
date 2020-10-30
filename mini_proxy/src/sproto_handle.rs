struct SprotoHandle{

}

impl SprotoHandle {

    pub fn Handle(spid: SprotoId){
        match spid {
            SProtoId::NewServer=> {

            },
            SProtoId::CloseSocket=> {

            },
            SProtoId::SocketClose=> {

            },
            SProtoId::BusyServer=> {

            },
            SProtoId::MsgQueueIsFull=>  {

            },
            SProtoId::ExceptionServer=> {

            },
            SProtoId::SocketIdNotExist=> {

            },
            _=> None,
        }
    }
}