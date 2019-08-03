use std::io::prelude::*;
use std::net::TcpStream;
use std::time;
use std::env;

fn main() {
   let mut args = env::args();

   args.next();

   let ip = match args.next() {
      Some(x) => x,
      None => {
         println!("Unable to fetch IP from args.");
         return;
      },
   };

   let port = match args.next() {
      Some(x) => x,
      None => {
         println!("Unable to fetch port from args.");
         return;
      },
   };

   let mut stream = match TcpStream::connect(format!("{}:{}", ip, port)) {
      Ok(x) => {
         println!("Connected.\n");
         x
      },
      Err(m) => {
         println!("Error while connecting: {}", m);
         return;
      }
   };

   stream.set_read_timeout(Some(time::Duration::from_millis(500)))
      .expect("Could not set read timeout.\n");

   loop {
      let mut buf = [0; 128];
      let size: usize = match stream.read(&mut buf) {
         Ok(x) => x,
         Err(_x) => {
            println!("No data retrieved.");
            0
         },
      };

      if size == 0 {
         continue;
      }

      println!("Size: {}\n", size);
   }
}
