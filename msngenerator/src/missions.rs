use std::env;
use std::fs::File; 
use std::io::Write;
use std::path::Path;

fn create_missions(num_msns: i64) -> Vec<String> {
   let arr_char = vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
   let arr_num = vec!["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

   let mut missions: Vec<String> = Vec::new();

   assert!(num_msns <= 1_000_000, "The max number of missions is 1,000,000.");

   for i in 0..num_msns {
      let j = (( i / 100_000 ) % 10) as usize;
      let k = (( i / 10_000 ) % 10) as usize;
      let l = (( i / 1_000) % 10) as usize;
      let m = (( i / 100 ) % 10) as usize;
      let n = (( i / 10 ) % 10) as usize;
      let o = (i % 10) as usize;
      missions.push(format!("{}{}{}{}{}{}", arr_char[j], arr_char[k], arr_char[l],
         arr_char[m], arr_num[n], arr_num[o]));
   }

   missions
}

fn create_ato(missions: Vec<String>, day: i64) -> String {
   let header: String = format!("EXER/TNG//
MSGID/ATO/LAB/ATOORB{}/MAR/CHG//
AKNLDG/NO//", day);

   let timefram: String = format!("TIMEFRAM/FROM:{0:02}0001ZFEB2012/TO:{0:02}2359ZFEB2012//", day);

   let header_pt2: &str = "HEADING/TASKING//
TSKCNTRY/US//
SVCTASK/A//
TASKUNIT/66 BAD/ICAO:KMER//";

   let footer: &str = "DECL/ORIG:SOURCE/15G/-/X7//";

   let mut num = 0;
   let mut index = 0;
   let hours = [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];
   let minute = 10;

   let msnmessages: Vec<String> = missions.iter().map(|msn| {
      num += 1;
      let hour = hours[index];

      let mut end_minute = 0;

      let end_ind = {
         if (index + 3) > 19 {
            end_minute = 30;
            19
         }
         else {
            index + 3
         }
      };

      index = (index + 1) % 20;
      let end_hour = hours[end_ind];

      format!(
"AMSNDAT/{0}/-/-/-/CAP/-/15M/DEPLOC:KMER/ARRLOC:KMER//
MSNACFT/1/ACTYP:MIG29/{1}/BEST/-/101/2{0:04o}/3{0:04o}//
AMSNLOC/{6:02}{2:02}{3:02}ZFEB/{6:02}{4:02}{5:02}ZFEB/A10//",
         num, msn, hour, minute, end_hour, end_minute, day)
   }).collect();

   let retval = format!("{}\n{}\n{}\n{}\n{}", header, timefram, header_pt2, msnmessages.join("\n"),
      footer);

   retval
}

pub fn run_mission_maker() {
   let mut args = env::args();

   args.next();

   let filename = args.next().unwrap();
   let day_count = args.next().unwrap().parse::<i64>().unwrap();

   //TODO: Allow for > 28 days by adjusting month.
   if day_count > 28 {
      println!("Day count must be 28 or less.");
      return;
   }

   let num_missions = args.next().unwrap().parse::<i64>().unwrap();

   let num_missions_per_day = num_missions / day_count;

   for i in 1..=day_count {
      let filepath = format!("{}{}.ato", filename, i);
      let path = Path::new(&filepath);
      let display = path.display();

      let mut file = match File::create(&path) {
         Err(y) => panic!("Could not create {}: {}", display, y),
         Ok(file) => file,
      };

      let missions: Vec<String> = create_missions(num_missions_per_day);

      let output: String = create_ato(missions, i);

      match file.write_all(output.as_bytes()) {
         Err(y) => panic!("Could not write to {}: {}", display, y),
         Ok(_) => {
            println!("Created file {}.", display);
            println!("Contains {} missions.\n", num_missions_per_day);
         },
      }
   }
}
