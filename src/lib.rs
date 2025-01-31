// use winit::window::Window;

pub mod window;
pub mod renderer;
pub mod mesh;
pub mod scene;

pub mod color_normal_vertex;
pub mod color_vertex;

pub mod test_assets;

pub mod ui_manager;

pub mod game_program;



pub use renderer::{instances::InstanceHandle, gpu_resources::MeshHandle, Renderer};
pub use mesh::{Mesh, Vertex};
pub use scene::{Transform, camera::Camera};
pub use scene::light::{AmbientLight, DirectionalLight};
pub use color_normal_vertex::ColorNormalVertex;
pub use game_program::{Game, GameProgram};
pub use window::{game_loop::GameLoop, run_program};
pub use ui_manager::UIManager;