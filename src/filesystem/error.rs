use std::error::Error;
use std::fmt;

// implementing my own error type to make it `Send`
// (therefore transmissible between threads)
// (for `async fn`s)

#[derive(Debug)]
pub struct MyError {
    message: String,
}

impl MyError {
    pub fn new(message: &str) -> MyError {
        MyError {
            message: message.to_string(),
        }
    }


}

impl Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[macro_export]
macro_rules! my_eprintln {
    () => {
        eprint!("\nLine {} in {}", line!(), file!()); // Forward the captured arguments to eprintln
        eprintln!(); // Just call eprintln without any custom processing
    };
    ($($arg:tt)*) => {
        // Add your custom processing here if needed
        eprint!("\nLine {} in {} : ", line!(), file!()); // Forward the captured arguments to eprintln
        eprintln!($($arg)*); // Forward the captured arguments to eprintln
    };
}