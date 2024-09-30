use echoes_graphics::example::RenderGame;


pub fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    pollster::block_on(
        echoes_graphics::window::run_program::<echoes_graphics::example::EchoesProgram<RenderGame>>()
    );
}