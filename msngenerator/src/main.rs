mod missions;

use std::time::Instant;

fn main() {
   let start = Instant::now();

   missions::run_mission_maker();

   let time = start.elapsed();

   println!("\nExecution Complete:");
   println!("Elapsed Time:");
   println!("   {} secs", time.as_secs());
   println!("   {} ms", time.subsec_millis());
   println!("   {} us", time.subsec_micros() - time.subsec_millis() * 1_000);
   println!("   {} ns", time.subsec_nanos() - time.subsec_micros() * 1_000);
}
