use rs7_parser::parse_message;
use rs7_terser::Terser;

fn main() {
    let hl7 = "MSH|^~\&|SendApp|SendFac|RecApp|RecFac|20240315||ADT^A01|12345|P|2.5\r\
               PID|1||MRN123|||DOE^JOHN||19800101|M";
    
    let message = parse_message(hl7).unwrap();
    let terser = Terser::new(&message);
    
    for i in 1..=10 {
        println!("PID-{}: {:?}", i, terser.get(&format!("PID-{}", i)));
    }
}
