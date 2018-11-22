mod accounts;
mod auth;
mod email_sender;
mod error;
mod rabbit;
mod transactions;
mod users;

pub use self::accounts::*;
pub use self::auth::*;
pub use self::email_sender::*;
pub use self::error::*;
pub use self::rabbit::*;
pub use self::transactions::*;
pub use self::users::*;

use prelude::*;

type ServiceFuture<T> = Box<Future<Item = T, Error = Error> + Send>;
