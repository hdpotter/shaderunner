use shaderunner::example::RenderGame;


pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    pollster::block_on(
        shaderunner::window::run_program::<shaderunner::example::EchoesProgram<RenderGame>>()
    );
}