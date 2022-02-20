/// image_1 & image_2 will be paths to each image file
#[derive(Debug)]
pub struct Args {
    // Fields need to be public to be accessible outside of module
    pub image_1: String,
    pub image_2: String,
    pub output: String
}

pub fn get_nth_arg(n: usize) -> String {
    std::env::args().nth(n).unwrap()
}

impl Args {
    pub fn new() -> Self {
        Args {
            image_1: get_nth_arg(1),
            image_2: get_nth_arg(2),
            output: get_nth_arg(3)
        }
    }
}
