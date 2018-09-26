use hyper;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

pub type Client = hyper::Client<HttpsConnector<HttpConnector>>;
