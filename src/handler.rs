use crate::{Context, Response};
use hyper::StatusCode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::time::{SystemTime};
use handlebars::Handlebars;
use serde_json::json;



//Structs ----------------------------------------------------------------------------------------------------
#[derive(Debug)]
struct HydraError(String);

impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}

impl Error for HydraError {}



//Functions --------------------------------------------------------------------------------------------------
pub async fn resolve(ctx: Context) -> Response {
    //Get a current timestamp
    let timestamp: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    println!("{:?} {:#?}", timestamp, ctx);

    //Get query parameters
    let params: HashMap<String, String> = ctx.req.uri().query().map(|v| {
        url::form_urlencoded::parse(v.as_bytes()).into_owned().collect()
    }).unwrap_or_else(HashMap::new);

    println!("{:#?}", params);

    //Get the real IP of the connecting client
    match ctx.req.headers().get("cf-connecting-ip") {
        Some(remote_ip) => {
            println!("\nREQUEST[CloudFlare]: {}", remote_ip.to_str().unwrap());
        }
        None => {
            println!("\nREQUEST: {}", ctx.state.remote_ip);
        }
    }

    //Give a landing page if a subdomain wasn't used
    if params.len() == 0 {
        let reg = Handlebars::new();
        let doc = fs::read_to_string("home.html").expect("Something went wrong reading the file.");
        let doc_rendered = reg.render_template(&doc, &json!({"version": ctx.state.version})).expect("Something went wrong rendering the file");
        return hyper::Response::builder()
            .status(StatusCode::OK)
            .body(format!("{}", doc_rendered).into())
            .unwrap();
    }

    //Return success all the time so we don't burden the outside world with
    //our own internal struggles :-)
    return hyper::Response::builder()
        .status(StatusCode::OK)
        .body(format!("Success!").into())
        .unwrap();
}
