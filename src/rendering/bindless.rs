use std::{mem, ptr};
use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint, GLuint64};
use mvutils::once::CreateOnce;

//special bindless textures opengl extension function pointers

pub static GetTextureHandleARB: CreateOnce<unsafe extern "C" fn(GLuint) -> GLuint64> = CreateOnce::new();
pub static GetTextureSamplerHandleARB: CreateOnce<unsafe extern "C" fn(GLuint, GLuint) -> GLuint64> = CreateOnce::new();
pub static MakeTextureHandleResidentARB: CreateOnce<unsafe extern "C" fn(GLuint64)> = CreateOnce::new();
pub static MakeTextureHandleNonResidentARB: CreateOnce<unsafe extern "C" fn(GLuint64)> = CreateOnce::new();
pub static GetImageHandleARB: CreateOnce<unsafe extern "C" fn(GLuint, GLint, GLboolean, GLint, GLenum) -> GLuint64> = CreateOnce::new();
pub static MakeImageHandleResidentARB: CreateOnce<unsafe extern "C" fn(GLuint64, GLenum)> = CreateOnce::new();
pub static MakeImageHandleNonResidentARB: CreateOnce<unsafe extern "C" fn(GLuint64)> = CreateOnce::new();
pub static IsTextureHandleResidentARB: CreateOnce<unsafe extern "C" fn(GLuint64) -> GLboolean> = CreateOnce::new();
pub static IsImageHandleResidentARB: CreateOnce<unsafe extern "C" fn(GLuint64) -> GLboolean> = CreateOnce::new();
pub static UniformHandleui64ARB: CreateOnce<unsafe extern "C" fn(GLint, GLuint64)> = CreateOnce::new();
pub static UniformHandleui64vARB: CreateOnce<unsafe extern "C" fn(GLint, GLsizei, *const GLuint64)> = CreateOnce::new();
pub static ProgramUniformHandleui64ARB: CreateOnce<unsafe extern "C" fn(GLuint, GLint, GLuint64)> = CreateOnce::new();
pub static ProgramUniformHandleui64vARB: CreateOnce<unsafe extern "C" fn(GLuint, GLint, GLsizei, *const GLuint64)> = CreateOnce::new();

unsafe fn load_function<T>(handle: &glutin::Window, name: &str) -> T {
    let pointer = handle.get_proc_address(name);
    if pointer.is_null() {
        panic!("Failed to load OpenGL function: {}", name);
    }
    mem::transmute_copy(&pointer)
}

pub unsafe fn load_bindless_texture_functions(handle: &glutin::Window) {
    GetTextureHandleARB.create(|| load_function(handle, "glGetTextureHandleARB"));
    GetTextureSamplerHandleARB.create(|| load_function(handle, "glGetTextureSamplerHandleARB"));
    MakeTextureHandleResidentARB.create(|| load_function(handle, "glMakeTextureHandleResidentARB"));
    MakeTextureHandleNonResidentARB.create(|| load_function(handle, "glMakeTextureHandleNonResidentARB"));
    GetImageHandleARB.create(|| load_function(handle, "glGetImageHandleARB"));
    MakeImageHandleResidentARB.create(|| load_function(handle, "glMakeImageHandleResidentARB"));
    MakeImageHandleNonResidentARB.create(|| load_function(handle, "glMakeImageHandleNonResidentARB"));
    IsTextureHandleResidentARB.create(|| load_function(handle, "glIsTextureHandleResidentARB"));
    IsImageHandleResidentARB.create(|| load_function(handle, "glIsImageHandleResidentARB"));
    UniformHandleui64ARB.create(|| load_function(handle, "glUniformHandleui64ARB"));
    UniformHandleui64vARB.create(|| load_function(handle, "glUniformHandleui64vARB"));
    ProgramUniformHandleui64ARB.create(|| load_function(handle, "glProgramUniformHandleui64ARB"));
    ProgramUniformHandleui64vARB.create(|| load_function(handle, "glProgramUniformHandleui64vARB"));
}
