// use log::info;


#[no_mangle]
pub extern "C" fn test_function1(num1: i32, num2: i32) -> i32 {
  // println!("Test wasm file run");
  // 42
  num1 + num2
}

// fn main() {
//   info!("main");
// }


/*
  TODO
    Run module asynchronously
      Using Webworker in wasm
      Example online demo is not working, which is weird
        Tried to run it locally, encountered errors as well
*/