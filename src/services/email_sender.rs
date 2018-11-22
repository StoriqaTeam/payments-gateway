use std::sync::Arc;

use handlebars::Handlebars;

use super::error::*;
use models::*;
use prelude::*;
use rabbit::TransactionPublisher;
use repos::{DbExecutor, TemplatesRepo};

pub trait EmailSenderService: Send + Sync + 'static {
    fn send_add_device(
        &self,
        email: String,
        token: DeviceConfirmToken,
        device_id: DeviceId,
    ) -> Box<Future<Item = (), Error = Error> + Send>;
    fn get_template(&self, name: TemplateName) -> Box<Future<Item = String, Error = Error> + Send>;
}

pub struct EmailSenderServiceImpl<E: DbExecutor> {
    templates_repo: Arc<dyn TemplatesRepo>,
    db_executor: E,
    publisher: Arc<dyn TransactionPublisher>,
    device_confirm_url: String,
}

impl<E: DbExecutor> EmailSenderServiceImpl<E> {
    pub fn new(
        templates_repo: Arc<dyn TemplatesRepo>,
        db_executor: E,
        publisher: Arc<dyn TransactionPublisher>,
        device_confirm_url: String,
    ) -> Self {
        EmailSenderServiceImpl {
            templates_repo,
            db_executor,
            publisher,
            device_confirm_url,
        }
    }
}

impl<E: DbExecutor> EmailSenderService for EmailSenderServiceImpl<E> {
    fn send_add_device(
        &self,
        email: String,
        token: DeviceConfirmToken,
        device_id: DeviceId,
    ) -> Box<Future<Item = (), Error = Error> + Send> {
        let device_confirm_url = self.device_confirm_url.clone();
        let publisher = self.publisher.clone();
        let mail = DeviceAddEmail::new(email, device_confirm_url, token, device_id);
        let handlebars = Handlebars::new();
        Box::new(
            self.get_template(TemplateName::AddDevice)
                .and_then({
                    let mail = mail.clone();
                    move |template| {
                        handlebars
                            .render_template(&template, &mail)
                            .map_err(ectx!(ErrorContext::RenderTemplate, ErrorKind::Internal => mail))
                            .into_future()
                    }
                }).and_then(move |text| {
                    let email = Email::new(mail.to, "New device will be added to your account".to_string(), text);
                    publisher
                        .send_email(email.clone())
                        .map_err(ectx!(ErrorContext::Lapin, ErrorKind::Internal => email))
                }),
        )
    }
    fn get_template(&self, name: TemplateName) -> Box<Future<Item = String, Error = Error> + Send> {
        let templates_repo = self.templates_repo.clone();
        let db_executor = self.db_executor.clone();
        Box::new(db_executor.execute(move || {
            let name_clone = name.clone();
            let template = templates_repo.get(name.clone()).map_err(ectx!(try convert => name))?;
            let t = template.ok_or_else(|| ectx!(try err ErrorContext::NoTemplate, ErrorKind::Internal => name_clone))?;
            Ok(t.data)
        }))
    }
}
