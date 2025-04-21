pub fn gl_check_errors() {
    let mut err = unsafe { gl::GetError() };
    while err != gl::NO_ERROR {
        match err {
            gl::INVALID_ENUM => {
                eprintln!("enumeration parameter is not a legal enumeration for that function");
            }
            gl::INVALID_VALUE => {
                eprintln!("value parameter is not a legal value for that function");
            }
            gl::INVALID_OPERATION => {
                eprintln!("the set of state for a command is not legal for the parameters given to that command");
            }
            gl::STACK_OVERFLOW => {
                eprintln!("stack pushing operation cannot be done because it would overflow the limit of that stack's size");
            }
            gl::STACK_UNDERFLOW => {
                eprintln!("stack popping operation cannot be done because the stack is already at its lowest point");
            }
            gl::OUT_OF_MEMORY => {
                eprintln!("performing an operation that can allocate memory, and the memory cannot be allocated");
            }
            gl::INVALID_FRAMEBUFFER_OPERATION => {
                eprintln!("doing anything that would attempt to read from or write/render to a framebuffer that is not complete");
            }
            gl::CONTEXT_LOST => {
                eprintln!("OpenGL context has been lost, due to a graphics card reset");
            }
            _ => {}
        };
        err = unsafe { gl::GetError() };
    }
}

pub struct GlAttrib {
    pub r#type: gl::types::GLenum,
    pub comps: gl::types::GLint,
    pub normalized: gl::types::GLboolean,
    pub stride: gl::types::GLsizei,
    pub offset: usize,
}
