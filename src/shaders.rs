use gl::types::{GLboolean, GLchar, GLenum, GLint, GLuint};
use std::ffi::CString;
use std::fs;

fn check_error(shader: GLuint, flag: GLuint, is_program: bool) -> Result<(), String> {
    let mut success: GLint = 0;
    let mut error: [GLchar; 1024] = [0; 1024];

    if is_program {
        unsafe {
            gl::GetProgramiv(shader, flag, &mut success);
        }
    } else {
        unsafe {
            gl::GetShaderiv(shader, flag, &mut success);
        }
    }

    if success as GLboolean == gl::FALSE {
        if is_program {
            unsafe {
                gl::GetProgramInfoLog(
                    shader,
                    error.len() as i32,
                    std::ptr::null_mut(),
                    &mut error[0] as *mut i8,
                );
            }
        } else {
            unsafe {
                gl::GetShaderInfoLog(
                    shader,
                    error.len() as i32,
                    std::ptr::null_mut(),
                    &mut error[0] as *mut i8,
                );
            }
        }

        let err_vec = error
            .into_iter()
            .map(|i| i as u8)
            .take_while(|ch| *ch != 0) // \0 from C
            .collect();

        let err_cstring = unsafe { CString::from_vec_unchecked(err_vec) };

        return Err(err_cstring.into_string().unwrap());
    }

    Ok(())
}

fn create_shader(text: &str, shader_type: GLenum) -> Result<GLuint, String> {
    let shader = unsafe { gl::CreateShader(shader_type) };

    if shader == 0 {
        return Err("Error: Shader creation failed".into());
    }

    let shader_source_strings: [*const GLchar; 1] = [text.as_ptr() as *const GLchar];
    let shader_source_string_lengths: [GLint; 1] = [text.len() as GLint];

    unsafe {
        gl::ShaderSource(
            shader,
            shader_source_string_lengths.len() as i32,
            shader_source_strings.as_ptr(),
            shader_source_string_lengths.as_ptr(),
        );
        gl::CompileShader(shader);
    }

    check_error(shader, gl::COMPILE_STATUS, false)
        .map_err(|s| format!("Error: Shader compilation failed. {s}"))?;

    Ok(shader)
}

pub fn load(vertex_file_path: &str, fragment_file_path: &str) -> Result<GLuint, String> {
    let vertex_shader_code =
        fs::read_to_string(vertex_file_path).map_err(|io_err| io_err.to_string())?;
    let fragment_shader_code =
        fs::read_to_string(fragment_file_path).map_err(|io_err| io_err.to_string())?;

    let vertex_shader_id = create_shader(&vertex_shader_code, gl::VERTEX_SHADER)?;
    let fragment_shader_id = create_shader(&fragment_shader_code, gl::FRAGMENT_SHADER)?;

    let program_id = unsafe { gl::CreateProgram() };

    unsafe {
        gl::AttachShader(program_id, vertex_shader_id);
        gl::AttachShader(program_id, fragment_shader_id);
    }

    unsafe {
        gl::LinkProgram(program_id);
    }
    check_error(program_id, gl::LINK_STATUS, true)
        .map_err(|s| format!("Error: Program linking failed. {s}"))?;

    unsafe {
        gl::ValidateProgram(program_id);
    }
    check_error(program_id, gl::VALIDATE_STATUS, true)
        .map_err(|s| format!("Error: Program validation failed. {s}"))?;

    unsafe {
        gl::DetachShader(program_id, vertex_shader_id);
        gl::DetachShader(program_id, fragment_shader_id);
    }

    unsafe {
        gl::DeleteShader(vertex_shader_id);
        gl::DeleteShader(fragment_shader_id);
    }

    Ok(program_id)
}
