use crate::{Context, Response};
use hyper::StatusCode;
use std::error::Error;
use std::fmt;
use std::time::{SystemTime};


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

    //println!("{:?} {:#?}", timestamp, ctx);

    //Get the real IP of the connecting client
    let client_ip;
    match ctx.req.headers().get("cf-connecting-ip") {
        Some(remote_ip) => {
            client_ip = remote_ip.to_str().unwrap();
        }
        None => {
            client_ip = ctx.state.remote_ip.as_str();
        }
    }

    //Get the host header so we can calculate a subdomain
    let subdomain: String;
    match ctx.req.headers().get("host") {
        Some(host) => {
            println!("\nREQUEST[host]: {}", host.to_str().unwrap());
            subdomain = host.to_str().unwrap().to_string();
        }
        None => {
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("Host header is required.").into())
                .unwrap();
        }
    }

    //The first member of the host header should be the guid we are to look for.  Take it
    //and strip out any dashes so that we match on both dashed and non-dashed submissions
    match subdomain.find(".guid.podcastindex.org") {
        Some(ending_at) => {

            //Strip dashes out of the guid requested if there are any
            let guid = subdomain[0..ending_at]
                .to_string()
                .replace("-", "")
                .replace("\"", "");

            //Lookup the url
            let url = ctx.state.guids.get(&guid);

            //Give back the url for http response if found
            if url.is_some() {
                //Give some logging
                println!("{:?}({:?}): {:?} -> {:#?}", client_ip, timestamp, guid, url.unwrap());

                //Return 200 with plain text url
                return hyper::Response::builder()
                    .status(StatusCode::OK)
                    .body(format!("{}", url.unwrap()).into())
                    .unwrap();
            }

            //The guid was not found in the hash table
            return hyper::Response::builder()
                .status(StatusCode::from_u16(404).unwrap())
                .body(format!("Guid: {:?} not found", guid).into())
                .unwrap();
        }
        None => {
            eprintln!("No guid found in host header.");
            return hyper::Response::builder()
                .status(StatusCode::from_u16(400).unwrap())
                .body(format!("Required host header format is: [guid].guid.podcastindex.org").into())
                .unwrap();
        }
    }
}