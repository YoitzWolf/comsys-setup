use std::cmp::max;
use std::{collections::HashMap, sync::Arc};
use crate::gen::comp_handler::{EqHistoryMessage, EqMessage, Verification};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{Mutex, mpsc};
use tonic::Status;


impl From<(i32, EqMessage)> for EqHistoryMessage {
    fn from(value: (i32, EqMessage)) -> Self {
        Self { message_id: value.0, message: Some(value.1) }
    }
}

type MessageSender = Sender<Result<EqHistoryMessage, Status>>;

#[derive(Debug)]
pub struct MessagePool {
    history: Arc<Mutex< Vec<EqHistoryMessage>>>,
    not_verified: Arc<Mutex< Vec<i32> >>,
    senders: Arc<Mutex< Vec<MessageSender>>>,
}

impl MessagePool {

    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(vec![])),
            not_verified: Arc::new(Mutex::new(vec![])),
            senders: Arc::new(Mutex::new(vec![])),
        }
    }

    pub async fn not_verified(&self, i: i32 ) -> bool {
        self.not_verified.lock().await.contains(&i)
    }

    pub async fn remove_from_unverifyed(&self, i: i32) -> Result<i32, Status> {
       {
        let mut dat = self.not_verified.lock().await;
        let pos = dat.iter().position(|x| x.eq(&i));
        match pos {
            Some(pos) => {
                Ok(dat.remove(pos))
            },
            None => Err(Status::not_found("Not found message error")),
        }
      }
    }

    pub async fn subscribe(&mut self, sender: MessageSender) {
        self.senders.lock().await.push(sender);
    }
    
    pub async fn send(&mut self, value: EqMessage) -> Result<usize, Status> {
        //let message = EqHistoryMessage::from((.len() as i32, value));
        //let res = self.sender.send(message.clone());
        //if let Ok(ok) = res {
        //    self.history.push(message)
        //}

        if value.message.is_none() {
            return Err(Status::invalid_argument("No message data"));
        }

        let message;
        let id: usize = {
            let mut history = self.history.lock().await;
            message = EqHistoryMessage::from((history.len() as i32, value.clone()));
            history.push(message.clone());
            history.len()
        };

        // Add Vote Message ID to list of unverified messages
        match value.message.unwrap() {
            crate::gen::comp_handler::eq_message::Message::VoteMessage(_) => {self.not_verified.lock().await.push(id as i32);}
            _ => {}
        };
            //

        {
            let mut senderlist = self.senders.lock().await;
            let mut bored = vec![];
            for (i, j) in senderlist.iter().enumerate() {
                match j.send(Ok(message.clone())).await {
                    Ok(_) => {/*println!("Sent message");*/},
                    Err(e) => {
                        /*println!(
                            "Bored: {:?}", e
                        );*/
                        bored.push(i);
                    },
                }
            }
            bored.iter().for_each(|x| drop(senderlist.remove(*x)) );
        };
        //println!("Sending finished");
        Ok(id)
    }

    pub async fn last_id(&self) -> i32 {
        self.history.lock().await.last().unwrap_or(&EqHistoryMessage{message_id:-1, message: None}).message_id
    }

    pub async fn index(&self, mid: i32) -> Option<EqHistoryMessage> {
        self.history.lock().await.get((mid as usize)).cloned()
    }

    /*pub fn last_msg(&self) -> Option<&EqHistoryMessage> {
        self.history.last().clone()
    }

    pub fn history_clone(&self) -> Vec<EqHistoryMessage> {
        self.history.clone()
    }

    pub fn nth_history_clone(&self, n:usize) -> Vec<EqHistoryMessage> {
        self.history[max(0, self.history.len() - n)..self.history.len()-1].to_vec()
    }*/

}