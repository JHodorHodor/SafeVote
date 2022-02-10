pub mod party;
pub mod circuit;
pub mod gate;
pub mod field;
pub mod share_receiver;
pub mod share_sender;
pub mod message;
mod polynomial;

#[cfg(test)]
mod tests {
    #[test]
    fn test_party_new() {
        env_logger::init();

        let rx = ShareReceiver { msg: None };
        let tx = ShareSender { chan: Rc::new(RefCell::new(rx.clone())) };
        
        let party = super::party::Party::new(0, 1, Box::new(rx), vec![Box::new(tx)],
            super::field::Field::new(97),
            super::circuit::Circuit::new(super::gate::Gate::<u8>::new_input(0), 1),
            1);
        assert_eq!(party.run(), 1);
    }

    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct ShareReceiver {
        msg: Option<super::message::Message<u8>>
    }

    impl super::share_receiver::ShareReceiver<super::message::Message<u8>> for ShareReceiver {
        fn recv(&mut self) -> super::message::Message<u8> {
            (*self.msg.as_ref().unwrap()).clone()
        }
    }

    struct ShareSender {
        chan: Rc<RefCell<ShareReceiver>>
    }

    impl super::share_sender::ShareSender<super::message::Message<u8>> for ShareSender {
        fn send(&mut self, msg: super::message::Message<u8>) {
            self.chan.borrow_mut().msg = Some(msg);
        }
    }
}
