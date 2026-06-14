use opencl3::{
    command_queue::{CL_BLOCKING, CommandQueue},
    context::Context,
    device::{CL_DEVICE_TYPE_ALL, Device},
    kernel::Kernel,
    memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, ClMem},
    platform::get_platforms,
    program::Program,
};

use std::ptr;

use crate::{Particle, Particle32, Particle64, Result, TrackingBytecode, UfoError};

pub const BASE_CL: &str = include_str!("kernels/base.cl");
pub const INSTRUCTIONS_CL: &str = include_str!("kernels/instructions.cl");
pub const INTERPRETER_CL: &str = include_str!("kernels/interpreter.cl");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub index: usize,
    pub name: String,
}

pub fn list_devices() -> Result<Vec<DeviceInfo>> {
    let mut devices = Vec::new();
    for platform in get_platforms().map_err(|e| UfoError::OpenCl(e.to_string()))? {
        for id in platform
            .get_devices(CL_DEVICE_TYPE_ALL)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?
        {
            let device = Device::new(id);
            devices.push(DeviceInfo {
                index: devices.len(),
                name: device.name().map_err(|e| UfoError::OpenCl(e.to_string()))?,
            });
        }
    }
    Ok(devices)
}

pub fn build_interpreter_for_first_device(options: &str) -> Result<()> {
    let platform = get_platforms()
        .map_err(|e| UfoError::OpenCl(e.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| UfoError::OpenCl("no OpenCL platforms found".to_string()))?;
    let device_id = platform
        .get_devices(CL_DEVICE_TYPE_ALL)
        .map_err(|e| UfoError::OpenCl(e.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| UfoError::OpenCl("no OpenCL devices found".to_string()))?;
    let device = Device::new(device_id);
    let context = Context::from_device(&device).map_err(|e| UfoError::OpenCl(e.to_string()))?;
    let build_options = format!("-I src/kernels {options}");
    Program::create_and_build_from_source(&context, INTERPRETER_CL, &build_options)
        .map_err(UfoError::OpenCl)?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct TrackRunOptions {
    pub turns: u32,
    pub local_instruction_words: Option<usize>,
    pub build_options: String,
}

impl Default for TrackRunOptions {
    fn default() -> Self {
        Self {
            turns: 1,
            local_instruction_words: None,
            build_options: "-cl-fast-relaxed-math -cl-mad-enable -DUFO_QUIET".to_string(),
        }
    }
}

pub fn track_first_device(
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &TrackRunOptions,
) -> Result<Vec<Particle>> {
    if tracking.bytecode.is_64bit() {
        let input = particles
            .iter()
            .copied()
            .map(Particle64::from)
            .collect::<Vec<_>>();
        let output = run_interpreter::<Particle64>(tracking, &input, options)?;
        Ok(output.into_iter().map(Particle::from).collect())
    } else {
        let input = particles
            .iter()
            .copied()
            .map(Particle32::from)
            .collect::<Vec<_>>();
        let output = run_interpreter::<Particle32>(tracking, &input, options)?;
        Ok(output.into_iter().map(Particle::from).collect())
    }
}

fn run_interpreter<T: Copy + Default>(
    tracking: &TrackingBytecode,
    input: &[T],
    options: &TrackRunOptions,
) -> Result<Vec<T>> {
    let platform = get_platforms()
        .map_err(|e| UfoError::OpenCl(e.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| UfoError::OpenCl("no OpenCL platforms found".to_string()))?;
    let device_id = platform
        .get_devices(CL_DEVICE_TYPE_ALL)
        .map_err(|e| UfoError::OpenCl(e.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| UfoError::OpenCl("no OpenCL devices found".to_string()))?;
    let device = Device::new(device_id);
    let context = Context::from_device(&device).map_err(|e| UfoError::OpenCl(e.to_string()))?;
    let queue = unsafe { CommandQueue::create(&context, device_id, 0) }
        .map_err(|e| UfoError::OpenCl(e.to_string()))?;

    let mut build_options = format!("-I src/kernels {}", options.build_options);
    if tracking.bytecode.is_64bit() {
        build_options.push_str(" -DUFO64");
    } else {
        build_options.push_str(" -cl-single-precision-constant");
    }
    let program = Program::create_and_build_from_source(&context, INTERPRETER_CL, &build_options)
        .map_err(UfoError::OpenCl)?;
    let kernel = Kernel::create(&program, "run").map_err(|e| UfoError::OpenCl(e.to_string()))?;

    let word_size = tracking.bytecode.word_size();
    let inst_buf_words = options
        .local_instruction_words
        .unwrap_or_else(|| tracking.bytecode.word_count().max(256));
    let bytecode = if inst_buf_words < tracking.bytecode.word_count() {
        tracking.bytecode.chunked(inst_buf_words)?
    } else {
        tracking.bytecode.clone()
    };
    let mut inst_bytes = bytecode.emit_bytes()?;
    let local_inst_buf_bytes = align_up(inst_buf_words * word_size, 64);
    let padded_inst_words = align_up(bytecode.word_count(), inst_buf_words);
    inst_bytes.resize(padded_inst_words * word_size, 0);

    let turns = options.turns.max(1);
    let output_len = input.len() * tracking.dumps_per_turn * turns as usize;
    let mut output = vec![T::default(); output_len];

    let mut input_buffer =
        unsafe { Buffer::<T>::create(&context, CL_MEM_READ_ONLY, input.len(), ptr::null_mut()) }
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
    let output_buffer =
        unsafe { Buffer::<T>::create(&context, CL_MEM_WRITE_ONLY, output.len(), ptr::null_mut()) }
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
    let mut inst_buffer = unsafe {
        Buffer::<u8>::create(
            &context,
            CL_MEM_READ_WRITE,
            inst_bytes.len(),
            ptr::null_mut(),
        )
    }
    .map_err(|e| UfoError::OpenCl(e.to_string()))?;

    unsafe {
        queue
            .enqueue_write_buffer(&mut input_buffer, CL_BLOCKING, 0, input, &[])
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        queue
            .enqueue_write_buffer(&mut inst_buffer, CL_BLOCKING, 0, &inst_bytes, &[])
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;

        let input_mem = input_buffer.get();
        let output_mem = output_buffer.get();
        let inst_mem = inst_buffer.get();
        let particles = input.len() as u32;
        let inst_buf_size = inst_buf_words as u32;
        let output_size = output_len as u32;
        let instructions = (bytecode.instructions().len() as u32) * turns;

        kernel
            .set_arg(0, &input_mem)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(1, &output_mem)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(2, &inst_mem)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg_local_buffer(3, local_inst_buf_bytes)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(4, &particles)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(5, &inst_buf_size)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(6, &turns)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(7, &output_size)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        kernel
            .set_arg(8, &instructions)
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;

        let global = [input.len()];
        let local = [1usize];
        queue
            .enqueue_nd_range_kernel(
                kernel.get(),
                1,
                ptr::null(),
                global.as_ptr(),
                local.as_ptr(),
                &[],
            )
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        queue
            .enqueue_read_buffer(&output_buffer, CL_BLOCKING, 0, &mut output, &[])
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
        queue
            .finish()
            .map_err(|e| UfoError::OpenCl(e.to_string()))?;
    }

    Ok(output)
}

fn align_up(value: usize, align: usize) -> usize {
    value.div_ceil(align) * align
}
