extern crate iron;
extern crate router;
extern crate persistent;
//Iron Base
use iron::prelude::*;
use iron::status;
use iron::typemap::Key;
//Iron Middleware
use persistent::State;
use router::Router;

use std::time::{Instant, Duration};

use std::process::Command;

//Time required between blanks; security measure
const RATE_LIMIT:u64 = 30;
//Setup persistent timer
#[derive(Copy, Clone)]
pub struct LastVisit;
impl Key for LastVisit { type Value = Instant; }

fn blank_screen(){
    Command::new("sh")
    .arg("-c").arg("xset dpms force off")
    .output().expect("Failed to execute process");
}
fn mute_sound(){
    Command::new("sh")
    .arg("-c").arg("amixer -D pulse set Master toggle")
    .output().expect("Failed to execute process");
}
fn blank_server(req: &mut Request) -> IronResult<Response> {
    println!("{:?}", req);
    let mutex = req.get::<State<LastVisit>>().unwrap();
    {
        let last_visit = mutex.read().unwrap();
        let now = Instant::now();
        let since_last = now.duration_since(*last_visit);
        println!("Time elapsed since last blank: {:?}", since_last.as_secs());
        if since_last > Duration::new(RATE_LIMIT, 0) {
            blank_screen();
            mute_sound();
        }
    }
    {
        let mut last_visit = mutex.write().unwrap();
        *last_visit = Instant::now();
    }
    Ok(Response::with((status::Ok, "Msg Received!")))
}
fn main() {
    let mut router = Router::new();
    router.get("/", blank_server, "index");
    let now = Instant::now() - Duration::new(RATE_LIMIT, 0);
    let mut chain = Chain::new(router);
    chain.link(State::<LastVisit>::both(now));
    Iron::new(chain).http("192.168.1.150:51339").unwrap();
    println!("Listening on port 51339.");
}
