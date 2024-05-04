// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

fn main() {
    use derive_builder::Builder;

    #[derive(Builder)]
    pub struct Command {
        executable: String,
        #[builder(optional, each(arg))]
        args: Vec<String>,
        #[builder(each(env))]
        env: Vec<String>,
        #[builder(optional)]
        current_dir: Option<String>,
    }
}
