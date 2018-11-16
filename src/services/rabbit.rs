use std::sync::Arc;

use futures::future;
use serde::Deserialize;
use serde_json;

use super::error::*;
use models::*;
use prelude::*;
use rabbit::TransactionPublisher;
use repos::{AccountsRepo, DbExecutor, DevicesRepo, UsersRepo};

#[derive(Clone)]
pub struct Notificator<E: DbExecutor> {
    accounts_repo: Arc<dyn AccountsRepo>,
    users_repo: Arc<dyn UsersRepo>,
    devices_repo: Arc<dyn DevicesRepo>,
    db_executor: E,
    publisher: Arc<dyn TransactionPublisher>,
}

impl<E: DbExecutor> Notificator<E> {
    pub fn new(
        accounts_repo: Arc<dyn AccountsRepo>,
        users_repo: Arc<dyn UsersRepo>,
        devices_repo: Arc<dyn DevicesRepo>,
        db_executor: E,
        publisher: Arc<dyn TransactionPublisher>,
    ) -> Self {
        Self {
            accounts_repo,
            users_repo,
            devices_repo,
            db_executor,
            publisher,
        }
    }

    pub fn handle_message(&self, data: Vec<u8>) -> impl Future<Item = (), Error = Error> + Send {
        let service = self.clone();
        let publisher = self.publisher.clone();
        parse::<Transaction>(data)
            .into_future()
            .and_then(move |transaction| service.get_transaction_info(transaction))
            .and_then(move |(transaction, devices, callback)| {
                let mut futs = vec![];
                for device in devices {
                    let push = PushNotifications::new(device.device_id, device.device_os, transaction.clone().into());
                    futs.push(publisher.push(push));
                }
                if let Some((callback_url, account_id)) = callback {
                    let callback = Callback::new(
                        callback_url,
                        transaction.to_value.to_string(),
                        transaction.to_currency,
                        transaction.to.blockchain_address,
                        account_id,
                    );
                    futs.push(publisher.callback(callback))
                }
                future::join_all(futs)
                    .map_err(ectx!(ErrorContext::Lapin, ErrorKind::Internal))
                    .map(|_| ())
            })
    }

    fn get_transaction_info(
        &self,
        mut transaction: Transaction,
    ) -> Box<Future<Item = (Transaction, Vec<Device>, Option<(String, AccountId)>), Error = Error> + Send> {
        let accounts_repo = self.accounts_repo.clone();
        let devices_repo = self.devices_repo.clone();
        let users_repo = self.users_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(db_executor.execute({
            move || {
                let mut devices = vec![];
                let mut callback = None;
                for from in &mut transaction.from {
                    if let Some(account_id) = from.account_id {
                        let account = accounts_repo
                            .get(account_id)
                            .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                        if let Some(account) = account {
                            let user = users_repo
                                .get(account.user_id)
                                .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                            if let Some(user) = user {
                                from.owner_name = Some(user.get_full_name());
                            }
                        } else {
                            // if account, for example, was deleted
                            from.account_id = None;
                        }
                    }
                }
                if let Some(account_id) = transaction.to.account_id {
                    let account = accounts_repo
                        .get(account_id)
                        .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                    if let Some(account) = account {
                        callback = account.callback_url.map(|callback_url| (callback_url, account_id));
                        let user = users_repo
                            .get(account.user_id)
                            .map_err(ectx!(try ErrorKind::Internal => account_id))?;
                        if let Some(user) = user {
                            transaction.to.owner_name = Some(user.get_full_name());

                            let mut user_devices = devices_repo
                                .get_by_user_id(user.id)
                                .map_err(ectx!(try ErrorKind::Internal => user))?;
                            devices.append(&mut user_devices);
                        }
                    } else {
                        // if account, for example, was deleted
                        transaction.to.account_id = None;
                    }
                }
                Ok((transaction, devices, callback))
            }
        }))
    }
}

fn parse<T>(data: Vec<u8>) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de> + Send,
{
    let data_clone = data.clone();
    let string = String::from_utf8(data).map_err(|e| ectx!(try err e, ErrorContext::UTF8, ErrorKind::Internal => data_clone))?;
    serde_json::from_str(&string).map_err(ectx!(ErrorContext::Json, ErrorKind::Internal => string))
}
