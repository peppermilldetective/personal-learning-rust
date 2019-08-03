use std::fs;
use std::env;

fn main() {
   let mut args = env::args();

   args.next();

   let mut prev = 0;
   let file = match args.next() {
      Some(x) => {
         x
      },
      None => {
         println!("File name required.");
         return;
      }
   };

   println!("Size of {}:", file);

   loop {
      let metadata = match fs::metadata(&file) {
         Ok(x) => { x },
         Err(_x) => { 
            println!("\n---- Invalid handle ----");
            continue;
         }
      };

      if metadata.len() < prev {
         println!("\n---- New File ----");
      }

      print!("\r{} KB | {} B", metadata.len() / 1000, metadata.len());

      prev = metadata.len();
   }
}
