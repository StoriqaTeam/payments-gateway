mod auth;
mod error;
mod users;

pub use self::auth::*;
pub use self::error::*;
pub use self::users::*;

use prelude::*;

type ServiceFuture<T> = Box<Future<Item = T, Error = Error> + Send>;
