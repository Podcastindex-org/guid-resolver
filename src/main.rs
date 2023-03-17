use std::collections::HashMap;
//Uses -----------------------------------------------------------------------------------------------------------------
use hyper::{
    body::to_bytes,
    service::{make_service_fn, service_fn},
    Body, Request, Server,
};
use route_recognizer::Params;
use router::Router;
use std::sync::Arc;
use hyper::server::conn::AddrStream;
use csv;
use std::env;
use spinners::{Spinner, Spinners};


//Globals --------------------------------------------------------------------------------------------------------------
mod handler;
mod router;
type Response = hyper::Response<hyper::Body>;
type Error = Box<dyn std::error::Error + Send + Sync + 'static>;


//Structs --------------------------------------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct AppState {
    pub state_thing: String,
    pub remote_ip: String,
    pub version: String,
    pub guids: Arc<HashMap<String, String>>,
}

#[derive(Debug)]
pub struct Context {
    pub state: AppState,
    pub req: Request<Body>,
    pub params: Params,
    body_bytes: Option<hyper::body::Bytes>,
}

impl Context {
    pub fn new(state: AppState, req: Request<Body>, params: Params) -> Context {
        Context {
            state,
            req,
            params,
            body_bytes: None,
        }
    }

    pub async fn body_json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, Error> {
        let body_bytes = match self.body_bytes {
            Some(ref v) => v,
            _ => {
                let body = to_bytes(self.req.body_mut()).await?;
                self.body_bytes = Some(body);
                self.body_bytes.as_ref().expect("body_bytes was set above")
            }
        };
        Ok(serde_json::from_slice(&body_bytes)?)
    }
}


//Main -----------------------------------------------------------------------------------------------------------------
#[tokio::main]
async fn main() {

    //Get what version we are
    let version = env!("CARGO_PKG_VERSION");
    println!("Version: {}", version);
    println!("--------------------");

    //Load the guid lookup table
    let mut guid_table: HashMap<String, String> = HashMap::new();
    if load_guid_table(&mut guid_table).is_err() {
        eprintln!("Could not load the guid list from file.");
        std::process::exit(1);
    }
    let guids: Arc<HashMap<String, String>> = Arc::new(guid_table);

    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/", Box::new(handler::resolve));

    let shared_router = Arc::new(router);
    let new_service = make_service_fn(move |conn: &AddrStream| {
        let app_state = AppState {
            state_thing: some_state.clone(),
            remote_ip: conn.remote_addr().to_string().clone(),
            version: version.to_string(),
            guids: Arc::clone(&guids)
        };

        let router_capture = shared_router.clone();
        async {
            Ok::<_, Error>(service_fn(move |req| {
                route(router_capture.clone(), req, app_state.clone())
            }))
        }
    });

    let addr = "0.0.0.0:80".parse().expect("address creation works");
    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{}", addr);

    let _ = server.await;
}


//Functions ------------------------------------------------------------------------------------------------------------
async fn route(
    router: Arc<Router>,
    req: Request<hyper::Body>,
    app_state: AppState
) -> Result<Response, Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let resp = found_handler
        .handler
        .invoke(Context::new(app_state, req, found_handler.params))
        .await;
    Ok(resp)
}


//Take a csv list of guid/url combos and load them into a passed in by ref hashmap
//Return true/false on success or failure
fn load_guid_table(guid_table: &mut HashMap<String, String>) -> Result<bool, Error> {
    let mut rdr = csv::Reader::from_path("guids.csv")?;

    //Show spinner on terminal while this loads cause it's big
    let mut sp = Spinner::new(Spinners::Dots9, "Loading guid records...".into());

    //Load each csv record into the hashmap
    for result in rdr.records() {
        let record = result?;

        let guid = record.get(1).unwrap().to_string().replace("-", "");
        let url = record.get(2).unwrap().to_string();

        //println!("{:?} - {:?}", guid, url);
        guid_table.insert(
            guid,
            url
        );
    }

    //Stop the spinner
    sp.stop();
    println!("Done");

    Ok(true)
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}