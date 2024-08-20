use std::cmp::max;
use std::{collections::HashMap, sync::Arc};
use crate::gen::comp_handler::{EqMessage, EqHistoryMessage};

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{Mutex, broadcast};


impl From<(i32, EqMessage)> for EqHistoryMessage {
    fn from(value: (i32, EqMessage)) -> Self {
        Self { message_id: value.0, message: Some(value.1) }
    }
}


#[derive(Debug)]
pub struct MessagePool {
    history: Vec<EqHistoryMessage>,
    sender: Sender<EqHistoryMessage>,
}

impl MessagePool {

    pub fn new(sender: Sender<EqHistoryMessage>) -> Self {
        Self {
            history: vec![],
            sender,
        }
    }

    pub fn subscribe(&self) -> Receiver<EqHistoryMessage> {
        self.sender.subscribe()
    }
    
    pub fn send(&mut self, value: EqMessage) -> Result<usize, broadcast::error::SendError<EqHistoryMessage>> {
        let message = EqHistoryMessage::from((self.history.len() as i32, value));
        let res = self.sender.send(message.clone());
        if let Ok(ok) = res {
            self.history.push(message)
        }
        Ok(self.history.len())
    }

    pub fn last_id(&self) -> i32 {
        self.history.last().unwrap_or(&EqHistoryMessage{message_id:-1, message: None}).message_id
    }
    pub fn last_msg(&self) -> Option<&EqHistoryMessage> {
        self.history.last().clone()
    }

    pub fn history_clone(&self) -> Vec<EqHistoryMessage> {
        self.history.clone()
    }

    pub fn nth_history_clone(&self, n:usize) -> Vec<EqHistoryMessage> {
        self.history[max(0, self.history.len() - n)..self.history.len()-1].to_vec()
    }

}