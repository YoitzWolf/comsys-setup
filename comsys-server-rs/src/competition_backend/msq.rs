use std::cmp::max;
use std::{collections::HashMap, sync::Arc};
use crate::gen::comp_handler::eq_message::Message;
use crate::gen::comp_handler::{EqHistoryMessage, EqMessage, Verification, VoteMessage};

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

    pub async fn select_unverified_with_author(&self, author_id: i32) -> Vec<i32> {
        {
            let hist = self.history.lock().await;
            self.not_verified.lock().await.iter().filter(
                |x| {
                    if let Some(obj) = hist.get(**x as usize) {
                        author_id.eq(& obj.message.as_ref().unwrap().author.as_ref().unwrap().uid)
                    } else {
                        false
                    }
                }
            ).cloned().collect()
        }
    }

    pub async fn select_unverified_with_query(&self, quidu: usize) -> Vec<i32> {
        let quid: i32 = quidu.try_into().unwrap();
        {
            let hist = self.history.lock().await;
            self.not_verified.lock().await.iter().filter(
                |x| {
                    if let Some(obj) = hist.get(**x as usize) {
                        if let Message::VoteMessage(vote) = obj.message.as_ref().unwrap().message.as_ref().unwrap()  {
                            quid.eq(& vote.queue_id)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            ).cloned().collect()
        }
    }

    /*pub async fn can_vote(&self, author_id: i32, queue_id: i32, action: i32) -> bool {
        let v = self.select_unverified_with_author(author_id).await;
        if v.len() > 0 { false }
        else {
            let hist = self.history.lock().await;
            hist.iter().rev().filter(
                |x|{
                    match &x.message {
                        Some(m) => match &m.message {
                            Some(Message::VoteMessage(vote)) => {
                                vote.action_id.eq(&author_id) && vote.queue_id.eq(&queue_id)
                            },
                            Some(Message::VerifyMessage(verify))
                            _ => false,
                        },
                        None => false,
                    }
                }
            );
            false
        }
    }*/

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
            history.len() - 1
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
            bored.iter().rev().for_each(|x| drop(senderlist.remove(*x)) );
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

    pub async fn history_clone(&self) -> Vec<EqHistoryMessage> {
        self.history.lock().await.clone()
    }

    /*pub fn last_msg(&self) -> Option<&EqHistoryMessage> {
        self.history.last().clone()
    }

    

    pub fn nth_history_clone(&self, n:usize) -> Vec<EqHistoryMessage> {
        self.history[max(0, self.history.len() - n)..self.history.len()-1].to_vec()
    }*/

}