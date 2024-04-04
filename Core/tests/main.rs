use glam::Mat4;
use russimp::Matrix4x4;

fn main() {
    let mut a = Matrix4x4::default();
    a.a1 = 1.0;
    a.a2 = 2.0;
    a.a3 = 3.0;
    a.b4 = 10.0;
    a.c3 = 20.0;
    a.d2 = 0.6;

    let mut b = unsafe { std::mem::transmute::<Matrix4x4, Mat4>(a) };

    println!("{b}");

}