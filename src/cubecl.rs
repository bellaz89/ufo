use crate::{
    Instruction, OP_CAVITY, OP_DRIFT, OP_DUMP, OP_EDGE, OP_KICK, OP_QUADRUPOLE, OP_REWIND,
    OP_SBEND, OP_SET_APERTURE, OP_TEAPOT, OP_TRAN_LINEAR, OP_WIRE, Particle, Result,
    TrackingBytecode, UfoError,
};

use ::cubecl::prelude::*;

const PARTICLE_WORDS: usize = 8;
const PARTICLE_X: usize = 0;
const PARTICLE_Y: usize = 1;
const PARTICLE_Z: usize = 2;
const PARTICLE_PX: usize = 3;
const PARTICLE_PY: usize = 4;
const PARTICLE_DP: usize = 5;
const PARTICLE_PASSED: usize = 6;
const PARTICLE_ALIVE: usize = 7;
const HEADER_OP_SHIFT: u32 = 0;
const HEADER_FLAGS_SHIFT: u32 = 8;
const HEADER_AUX_SHIFT: u32 = 16;
const HEADER_ARGC_SHIFT: u32 = 24;
const HEADER_FIELD_MASK: u32 = 0xff;
const FLAG_EXACT: u32 = 0x1 << 2;
const FLAG_ACHROMATIC: u32 = 0x1 << 6;
const FLAG_NO_APERTURE_CHECK: u32 = 0x1 << 7;
const FLAG_TRAN_LINEAR_VEC: u32 = 0x1 << 0;
const FLAG_TRAN_LINEAR_MAT_XX: u32 = 0x1 << 1;
const FLAG_TRAN_LINEAR_MAT_PXX: u32 = 0x1 << 2;
const FLAG_TRAN_LINEAR_MAT_XPX: u32 = 0x1 << 3;
const FLAG_TRAN_LINEAR_MAT_PXPX: u32 = 0x1 << 4;
const FLAG_TRAN_LINEAR_DP: u32 = 0x1 << 5;
const FLAG_TRAN_LINEAR_NO_PASS: u32 = 0x1 << 6;
const CUBE_UNITS: u32 = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CubeClBackend {
    Auto,
    Cpu,
    Vulkan,
    Cuda,
    Hip,
    Metal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CubeClDevice {
    Default,
    Index(usize),
    WgpuCpu,
    DiscreteGpu(usize),
    IntegratedGpu(usize),
    VirtualGpu(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CubeClDeviceInfo {
    pub backend: CubeClBackend,
    pub device: CubeClDevice,
    pub selector: String,
}

#[derive(Clone, Debug)]
pub struct CubeClTrackRunOptions {
    pub turns: u32,
    pub backend: CubeClBackend,
    pub device: CubeClDevice,
}

impl Default for CubeClTrackRunOptions {
    fn default() -> Self {
        Self {
            turns: 1,
            backend: CubeClBackend::Auto,
            device: CubeClDevice::Default,
        }
    }
}

#[derive(Clone, Debug)]
struct DecodedProgram {
    headers: Vec<u32>,
    arg_offsets: Vec<u32>,
    args32: Vec<f32>,
    args64: Vec<f64>,
}

pub fn track(tracking: &TrackingBytecode, particles: &[Particle]) -> Result<Vec<Particle>> {
    track_with_options(tracking, particles, &CubeClTrackRunOptions::default())
}

pub fn track_with_options(
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>> {
    match options.backend {
        CubeClBackend::Auto => {
            #[cfg(feature = "cubecl-vulkan")]
            {
                if let Some(device) = first_wgpu_device::<::cubecl::wgpu::Vulkan>() {
                    return track_on_wgpu_device::<::cubecl::wgpu::Vulkan>(
                        device, tracking, particles, options,
                    );
                }
            }
            #[cfg(feature = "cubecl-cpu")]
            {
                return track_on_runtime::<::cubecl::cpu::CpuRuntime>(tracking, particles, options);
            }
        }
        CubeClBackend::Cpu => {
            #[cfg(feature = "cubecl-cpu")]
            {
                return track_on_runtime::<::cubecl::cpu::CpuRuntime>(tracking, particles, options);
            }
        }
        CubeClBackend::Vulkan => {
            #[cfg(feature = "cubecl-vulkan")]
            {
                return track_on_wgpu::<::cubecl::wgpu::Vulkan>(tracking, particles, options);
            }
        }
        CubeClBackend::Cuda => {
            #[cfg(feature = "cubecl-cuda")]
            {
                let device = match options.device {
                    CubeClDevice::Default => ::cubecl::cuda::CudaDevice::new(0),
                    CubeClDevice::Index(index) => ::cubecl::cuda::CudaDevice::new(index),
                    other => {
                        return Err(UfoError::CubeCl(format!(
                            "device selector {:?} is not valid for CUDA",
                            other
                        )));
                    }
                };
                return track_on_device::<::cubecl::cuda::CudaRuntime>(
                    device, tracking, particles, options,
                );
            }
        }
        CubeClBackend::Hip => {
            #[cfg(feature = "cubecl-hip")]
            {
                let device = match options.device {
                    CubeClDevice::Default => ::cubecl::hip::AmdDevice::new(0),
                    CubeClDevice::Index(index) => ::cubecl::hip::AmdDevice::new(index),
                    other => {
                        return Err(UfoError::CubeCl(format!(
                            "device selector {:?} is not valid for HIP",
                            other
                        )));
                    }
                };
                return track_on_device::<::cubecl::hip::HipRuntime>(
                    device, tracking, particles, options,
                );
            }
        }
        CubeClBackend::Metal => {
            #[cfg(feature = "cubecl-metal")]
            {
                return track_on_wgpu::<::cubecl::wgpu::Metal>(tracking, particles, options);
            }
        }
    }

    Err(UfoError::CubeCl(format!(
        "backend {:?} is not available",
        options.backend
    )))
}

pub fn list_devices() -> Vec<CubeClDeviceInfo> {
    let mut devices = Vec::new();

    #[cfg(feature = "cubecl-vulkan")]
    {
        devices.extend(list_wgpu_devices::<::cubecl::wgpu::Vulkan>(
            CubeClBackend::Vulkan,
            "vulkan",
        ));
    }
    #[cfg(feature = "cubecl-cpu")]
    {
        devices.push(CubeClDeviceInfo {
            backend: CubeClBackend::Cpu,
            device: CubeClDevice::Default,
            selector: "cpu:0".to_string(),
        });
    }
    #[cfg(feature = "cubecl-cuda")]
    {
        for id in <::cubecl::cuda::CudaRuntime as Runtime>::enumerate_all_devices(&()) {
            let index = id.index_id as usize;
            devices.push(CubeClDeviceInfo {
                backend: CubeClBackend::Cuda,
                device: CubeClDevice::Index(index),
                selector: format!("cuda:{index}"),
            });
        }
    }
    #[cfg(feature = "cubecl-hip")]
    {
        for id in <::cubecl::hip::HipRuntime as Runtime>::enumerate_all_devices(&()) {
            let index = id.index_id as usize;
            devices.push(CubeClDeviceInfo {
                backend: CubeClBackend::Hip,
                device: CubeClDevice::Index(index),
                selector: format!("hip:{index}"),
            });
        }
    }
    #[cfg(feature = "cubecl-metal")]
    {
        devices.extend(list_wgpu_devices::<::cubecl::wgpu::Metal>(
            CubeClBackend::Metal,
            "metal",
        ));
    }

    devices
}

#[cfg(any(feature = "cubecl-vulkan", feature = "cubecl-metal"))]
fn list_wgpu_devices<G: ::cubecl::wgpu::GraphicsApi>(
    backend: CubeClBackend,
    prefix: &str,
) -> Vec<CubeClDeviceInfo> {
    <::cubecl::wgpu::WgpuRuntime as Runtime>::enumerate_all_devices(&G::backend())
        .into_iter()
        .map(|id| {
            let device = wgpu_device_from_id(id.type_id, id.index_id as usize);
            CubeClDeviceInfo {
                backend,
                selector: format!("{prefix}:{}", device_selector_suffix(device)),
                device,
            }
        })
        .collect()
}

#[cfg(any(feature = "cubecl-vulkan", feature = "cubecl-metal"))]
fn first_wgpu_device<G: ::cubecl::wgpu::GraphicsApi>() -> Option<CubeClDevice> {
    <::cubecl::wgpu::WgpuRuntime as Runtime>::enumerate_all_devices(&G::backend())
        .into_iter()
        .next()
        .map(|id| wgpu_device_from_id(id.type_id, id.index_id as usize))
}

#[cfg(any(feature = "cubecl-vulkan", feature = "cubecl-metal"))]
fn wgpu_device_from_id(type_id: u16, index: usize) -> CubeClDevice {
    match type_id {
        0 => CubeClDevice::DiscreteGpu(index),
        1 => CubeClDevice::IntegratedGpu(index),
        2 => CubeClDevice::VirtualGpu(index),
        3 => CubeClDevice::WgpuCpu,
        _ => CubeClDevice::Default,
    }
}

#[cfg(any(feature = "cubecl-vulkan", feature = "cubecl-metal"))]
fn device_selector_suffix(device: CubeClDevice) -> String {
    match device {
        CubeClDevice::Default => "default".to_string(),
        CubeClDevice::Index(index) => index.to_string(),
        CubeClDevice::WgpuCpu => "cpu".to_string(),
        CubeClDevice::DiscreteGpu(index) => format!("discrete:{index}"),
        CubeClDevice::IntegratedGpu(index) => format!("integrated:{index}"),
        CubeClDevice::VirtualGpu(index) => format!("virtual:{index}"),
    }
}

#[cfg(feature = "cubecl-cpu")]
fn track_on_runtime<R: Runtime>(
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>>
where
    R::Device: Default,
{
    let client = R::client(&Default::default());
    track_on_client::<R>(client, tracking, particles, options)
}

#[cfg(any(feature = "cubecl-cuda", feature = "cubecl-hip"))]
fn track_on_device<R: Runtime>(
    device: R::Device,
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>> {
    let client = R::client(&device);
    track_on_client::<R>(client, tracking, particles, options)
}

fn track_on_client<R: Runtime>(
    client: ComputeClient<R>,
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>> {
    if tracking.bytecode.is_64bit() {
        run_interpreter::<R, f64>(client, tracking, particles, options)
    } else {
        run_interpreter::<R, f32>(client, tracking, particles, options)
    }
}

#[cfg(any(feature = "cubecl-vulkan", feature = "cubecl-metal"))]
fn track_on_wgpu<G: ::cubecl::wgpu::GraphicsApi>(
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>> {
    let device = match options.device {
        CubeClDevice::Default => first_wgpu_device::<G>()
            .ok_or_else(|| UfoError::CubeCl(format!("no {:?} device found", options.backend)))?,
        device => device,
    };
    track_on_wgpu_device::<G>(device, tracking, particles, options)
}

#[cfg(any(feature = "cubecl-vulkan", feature = "cubecl-metal"))]
fn track_on_wgpu_device<G: ::cubecl::wgpu::GraphicsApi>(
    device: CubeClDevice,
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>> {
    let device = match device {
        CubeClDevice::Default => ::cubecl::wgpu::WgpuDevice::DefaultDevice,
        CubeClDevice::WgpuCpu => ::cubecl::wgpu::WgpuDevice::Cpu,
        CubeClDevice::DiscreteGpu(index) => ::cubecl::wgpu::WgpuDevice::DiscreteGpu(index),
        CubeClDevice::IntegratedGpu(index) => ::cubecl::wgpu::WgpuDevice::IntegratedGpu(index),
        CubeClDevice::VirtualGpu(index) => ::cubecl::wgpu::WgpuDevice::VirtualGpu(index),
        CubeClDevice::Index(index) => ::cubecl::wgpu::WgpuDevice::DiscreteGpu(index),
    };
    ::cubecl::wgpu::init_setup::<G>(&device, ::cubecl::wgpu::RuntimeOptions::default());
    let client = <::cubecl::wgpu::WgpuRuntime as Runtime>::client(&device);
    track_on_client::<::cubecl::wgpu::WgpuRuntime>(client, tracking, particles, options)
}

trait HostFloat: Float + CubeElement + Sized {
    fn program_args(program: &DecodedProgram) -> &[Self];
    fn pack_particles(particles: &[Particle]) -> Vec<Self>;
    fn unpack_particles(raw: &[Self]) -> Vec<Particle>;
}

impl HostFloat for f32 {
    fn program_args(program: &DecodedProgram) -> &[Self] {
        &program.args32
    }

    fn pack_particles(particles: &[Particle]) -> Vec<Self> {
        particles.iter().flat_map(particle_words_f32).collect()
    }

    fn unpack_particles(raw: &[Self]) -> Vec<Particle> {
        raw.chunks_exact(PARTICLE_WORDS)
            .map(|chunk| Particle {
                x: chunk[PARTICLE_X] as f64,
                y: chunk[PARTICLE_Y] as f64,
                z: chunk[PARTICLE_Z] as f64,
                px: chunk[PARTICLE_PX] as f64,
                py: chunk[PARTICLE_PY] as f64,
                dp: chunk[PARTICLE_DP] as f64,
                passed_elements: chunk[PARTICLE_PASSED] as u32,
                alive: chunk[PARTICLE_ALIVE] != 0.0,
            })
            .collect()
    }
}

impl HostFloat for f64 {
    fn program_args(program: &DecodedProgram) -> &[Self] {
        &program.args64
    }

    fn pack_particles(particles: &[Particle]) -> Vec<Self> {
        particles.iter().flat_map(particle_words_f64).collect()
    }

    fn unpack_particles(raw: &[Self]) -> Vec<Particle> {
        raw.chunks_exact(PARTICLE_WORDS)
            .map(|chunk| Particle {
                x: chunk[PARTICLE_X],
                y: chunk[PARTICLE_Y],
                z: chunk[PARTICLE_Z],
                px: chunk[PARTICLE_PX],
                py: chunk[PARTICLE_PY],
                dp: chunk[PARTICLE_DP],
                passed_elements: chunk[PARTICLE_PASSED] as u32,
                alive: chunk[PARTICLE_ALIVE] != 0.0,
            })
            .collect()
    }
}

fn run_interpreter<R, F>(
    client: ComputeClient<R>,
    tracking: &TrackingBytecode,
    particles: &[Particle],
    options: &CubeClTrackRunOptions,
) -> Result<Vec<Particle>>
where
    R: Runtime,
    F: HostFloat,
{
    let program = decode_program(tracking.bytecode.instructions());
    let turns = options.turns.max(1);
    let output_len = particles.len() * tracking.dumps_per_turn * turns as usize;

    let input = F::pack_particles(particles);
    let output = vec![F::new(0.0); output_len * PARTICLE_WORDS];
    let args = F::program_args(&program);

    let input_handle = client.create_from_slice(F::as_bytes(&input));
    let output_handle = client.create_from_slice(F::as_bytes(&output));
    let headers_handle = client.create_from_slice(u32::as_bytes(&program.headers));
    let offsets_handle = client.create_from_slice(u32::as_bytes(&program.arg_offsets));
    let args_handle = client.create_from_slice(F::as_bytes(args));
    let cube_count = particles.len().div_ceil(CUBE_UNITS as usize) as u32;

    unsafe {
        run_kernel::launch::<F, R>(
            &client,
            CubeCount::Static(cube_count, 1, 1),
            CubeDim::new_1d(CUBE_UNITS),
            ArrayArg::from_raw_parts(input_handle, input.len()),
            ArrayArg::from_raw_parts(output_handle.clone(), output.len()),
            ArrayArg::from_raw_parts(headers_handle, program.headers.len()),
            ArrayArg::from_raw_parts(offsets_handle, program.arg_offsets.len()),
            ArrayArg::from_raw_parts(args_handle, args.len()),
            particles.len(),
            turns,
            output_len,
            program.headers.len(),
            args.len(),
        );
    }

    let bytes = client.read_one(output_handle).map_err(|err| {
        UfoError::CubeCl(format!("failed to read CubeCL tracking output: {err:?}"))
    })?;
    let raw = F::from_bytes(&bytes);
    Ok(F::unpack_particles(raw))
}

fn encode_header(instruction: &Instruction) -> u32 {
    ((instruction.op as u32) << HEADER_OP_SHIFT)
        | ((instruction.flags.bits() as u32) << HEADER_FLAGS_SHIFT)
        | ((instruction.aux as u32) << HEADER_AUX_SHIFT)
        | ((instruction.args.len() as u32) << HEADER_ARGC_SHIFT)
}

#[cfg(test)]
fn header_field(header: u32, shift: u32) -> u32 {
    (header >> shift) & HEADER_FIELD_MASK
}

#[cube]
fn header_field_cube(header: u32, shift: u32) -> u32 {
    (header >> shift) & HEADER_FIELD_MASK
}

fn decode_program(instructions: &[Instruction]) -> DecodedProgram {
    let mut headers = Vec::with_capacity(instructions.len());
    let mut arg_offsets = Vec::with_capacity(instructions.len());
    let mut args32 = Vec::new();
    let mut args64 = Vec::new();

    for instruction in instructions {
        headers.push(encode_header(instruction));
        arg_offsets.push(args32.len() as u32);
        args32.extend(instruction.args.iter().map(|value| *value as f32));
        args64.extend(instruction.args.iter().copied());
    }

    DecodedProgram {
        headers,
        arg_offsets,
        args32,
        args64,
    }
}

fn particle_words_f32(particle: &Particle) -> [f32; PARTICLE_WORDS] {
    [
        particle.x as f32,
        particle.y as f32,
        particle.z as f32,
        particle.px as f32,
        particle.py as f32,
        particle.dp as f32,
        particle.passed_elements as f32,
        particle.alive as u32 as f32,
    ]
}

fn particle_words_f64(particle: &Particle) -> [f64; PARTICLE_WORDS] {
    [
        particle.x,
        particle.y,
        particle.z,
        particle.px,
        particle.py,
        particle.dp,
        particle.passed_elements as f64,
        particle.alive as u32 as f64,
    ]
}

#[cube(launch)]
fn run_kernel<F: Float>(
    input: &Array<F>,
    output: &mut Array<F>,
    headers: &Array<u32>,
    arg_offsets: &Array<u32>,
    args: &Array<F>,
    particles: usize,
    turns: u32,
    output_size: usize,
    #[comptime] instruction_count: usize,
    #[comptime] arg_count: usize,
) {
    let mut cached_headers = SharedMemory::<u32>::new(instruction_count);
    let mut cached_arg_offsets = SharedMemory::<u32>::new(instruction_count);
    let mut cached_args = SharedMemory::<F>::new(arg_count);
    let mut load = UNIT_POS as usize;
    while load < instruction_count {
        cached_headers[load] = headers[load];
        cached_arg_offsets[load] = arg_offsets[load];
        load += CUBE_DIM_X as usize;
    }
    load = UNIT_POS as usize;
    while load < arg_count {
        cached_args[load] = args[load];
        load += CUBE_DIM_X as usize;
    }
    sync_cube();

    if ABSOLUTE_POS < particles {
        let idx = ABSOLUTE_POS as usize;
        let base = idx * PARTICLE_WORDS;
        let mut x = input[base + PARTICLE_X];
        let mut y = input[base + PARTICLE_Y];
        let z = input[base + PARTICLE_Z];
        let mut px = input[base + PARTICLE_PX];
        let mut py = input[base + PARTICLE_PY];
        let mut dp = input[base + PARTICLE_DP];
        let mut passed_elements = input[base + PARTICLE_PASSED];
        let mut alive = input[base + PARTICLE_ALIVE];
        let mut opdp = dp + F::new(1.0);
        let mut oodppo = F::new(1.0) / opdp;
        let mut aperture_sq = F::new(3.4028234663852886e38);
        let mut turn = 0u32;
        let mut inst_current = 0usize;
        let mut output_offset = 0usize;

        while turn < turns && output_offset < output_size && inst_current < instruction_count {
            let header = cached_headers[inst_current];
            let op = header_field_cube(header, HEADER_OP_SHIFT);
            let flags = header_field_cube(header, HEADER_FLAGS_SHIFT);
            let aux = header_field_cube(header, HEADER_AUX_SHIFT);
            let argc = header_field_cube(header, HEADER_ARGC_SHIFT);
            let arg_start = cached_arg_offsets[inst_current] as usize;
            inst_current += 1;

            if op == OP_DUMP as u32 {
                let out_base = (output_offset + idx) * PARTICLE_WORDS;
                output[out_base + PARTICLE_X] = x;
                output[out_base + PARTICLE_Y] = y;
                output[out_base + PARTICLE_Z] = z;
                output[out_base + PARTICLE_PX] = px;
                output[out_base + PARTICLE_PY] = py;
                output[out_base + PARTICLE_DP] = dp;
                output[out_base + PARTICLE_PASSED] = passed_elements;
                output[out_base + PARTICLE_ALIVE] = alive;
                output_offset += particles;
            } else if op == OP_DRIFT as u32 {
                drift::<F>(flags, cached_args[arg_start], &mut x, &mut y, px, py, opdp);
            } else if op == OP_KICK as u32 {
                kick::<F>(
                    flags,
                    aux,
                    argc - aux,
                    arg_start,
                    &cached_args,
                    x,
                    y,
                    oodppo,
                    &mut px,
                    &mut py,
                );
            } else if op == OP_TEAPOT as u32 {
                let inner = cached_args[arg_start];
                let outer = cached_args[arg_start + 1];
                let weak_coeff = cached_args[arg_start + 2];
                let slices = u32::cast_from(cached_args[arg_start + 3]);
                drift::<F>(flags, outer, &mut x, &mut y, px, py, opdp);
                let mut i = 0u32;
                while i < slices - 1 {
                    kick::<F>(
                        flags,
                        aux,
                        argc - aux - 4,
                        arg_start + 4,
                        &cached_args,
                        x,
                        y,
                        oodppo,
                        &mut px,
                        &mut py,
                    );
                    px -= x * weak_coeff;
                    drift::<F>(flags, inner, &mut x, &mut y, px, py, opdp);
                    i += 1;
                }
                kick::<F>(
                    flags,
                    aux,
                    argc - aux - 4,
                    arg_start + 4,
                    &cached_args,
                    x,
                    y,
                    oodppo,
                    &mut px,
                    &mut py,
                );
                px -= x * weak_coeff;
                drift::<F>(flags, outer, &mut x, &mut y, px, py, opdp);
            } else if op == OP_QUADRUPOLE as u32 {
                let k0 = cached_args[arg_start];
                let length = cached_args[arg_start + 1];
                let k = if flags & FLAG_ACHROMATIC != 0 {
                    k0
                } else {
                    k0 * oodppo
                };
                let x0 = x;
                let y0 = y;
                let px0 = px;
                let py0 = py;
                let k2 = k.abs().sqrt();
                let k2l = length * k2;
                if k > F::new(0.0) {
                    let c = k2l.cos();
                    let ch = k2l.cosh();
                    let s = k2l.sin();
                    let sh = k2l.sinh();
                    x = c * x0 + s * px0 / k2;
                    y = ch * y0 + sh * py0 / k2;
                    px = -s * x0 * k2 + c * px0;
                    py = sh * y0 * k2 + ch * py0;
                } else {
                    let c = k2l.cos();
                    let ch = k2l.cosh();
                    let s = k2l.sin();
                    let sh = k2l.sinh();
                    x = ch * x0 + sh * px0 / k2;
                    y = c * y0 + s * py0 / k2;
                    px = sh * x0 * k2 + ch * px0;
                    py = -s * y0 * k2 + c * py0;
                }
            } else if op == OP_SBEND as u32 {
                let length = cached_args[arg_start];
                let curvature_coeff = cached_args[arg_start + 1];
                let k1 = cached_args[arg_start + 2];
                let x0 = x;
                let y0 = y;
                let px0 = px;
                let py0 = py;
                let dp0 = dp;
                let curvature = if flags & FLAG_ACHROMATIC != 0 {
                    curvature_coeff
                } else {
                    curvature_coeff * oodppo
                };
                let mut k = if flags & FLAG_ACHROMATIC != 0 {
                    k1
                } else {
                    k1 * oodppo
                };
                if k == F::new(0.0) {
                    y += length * py0;
                } else {
                    let k2 = k.abs().sqrt();
                    let k2l = length * k2;
                    if k > F::new(0.0) {
                        let ch = k2l.cosh();
                        let sh = k2l.sinh();
                        y = ch * y0 + sh * py0 / k2;
                        py = sh * y0 * k2 + ch * py0;
                    }
                    if k < F::new(0.0) {
                        let c = k2l.cos();
                        let s = k2l.sin();
                        y = c * y0 + s * py0 / k2;
                        py = -s * y0 * k2 + c * py0;
                    }
                }
                k += if flags & FLAG_ACHROMATIC != 0 {
                    curvature * curvature
                } else {
                    curvature * curvature_coeff
                };
                if k == F::new(0.0) {
                    x += length * px0;
                } else {
                    let k2 = k.abs().sqrt();
                    let k2_recip = F::new(1.0) / k2;
                    let k2l = length * k2;
                    if k > F::new(0.0) {
                        let c = k2l.cos();
                        let s = k2l.sin();
                        x = c * x0 + s * px0 * k2_recip;
                        x += dp0 * curvature * (F::new(1.0) - c) / k.abs();
                        px = -s * x0 * k2 + c * px0;
                        px += dp0 * curvature * s * k2_recip;
                    }
                    if k < F::new(0.0) {
                        let ch = k2l.cosh();
                        let sh = k2l.sinh();
                        x = ch * x0 + sh * px0 * k2_recip;
                        x += dp0 * curvature * (ch - F::new(1.0)) / k.abs();
                        px = sh * x0 * k2 + ch * px0;
                        px += dp0 * curvature * sh * k2_recip;
                    }
                }
            } else if op == OP_EDGE as u32 {
                let angle_length_ratio = cached_args[arg_start];
                let e = cached_args[arg_start + 1];
                let tan_e = cached_args[arg_start + 2];
                let psi_coeff = cached_args[arg_start + 3];
                let curvature = if flags & FLAG_ACHROMATIC != 0 {
                    angle_length_ratio
                } else {
                    angle_length_ratio * oodppo
                };
                let psi = e - curvature * psi_coeff;
                px += x * curvature * tan_e;
                py -= y * curvature * psi.tan();
            } else if op == OP_WIRE as u32 {
                let k = cached_args[arg_start];
                let wire_x = cached_args[arg_start + 1];
                let wire_y = cached_args[arg_start + 2];
                let dx = wire_x - x;
                let dy = wire_y - y;
                let alpha = dy.atan2(dx);
                let b = k / (dx * dx + dy * dy).sqrt();
                px += b * alpha.cos();
                py += b * alpha.sin();
            } else if op == OP_CAVITY as u32 {
                let field = cached_args[arg_start];
                let omega = cached_args[arg_start + 1];
                let lag = cached_args[arg_start + 2];
                dp += field * (F::new(6.283185307179586) * (lag - omega * z)).sin();
                opdp = dp + F::new(1.0);
                oodppo = F::new(1.0) / opdp;
            } else if op == OP_TRAN_LINEAR as u32 {
                let x0 = x;
                let y0 = y;
                let px0 = px;
                let py0 = py;
                let dp0 = dp;
                let mut slice = 0usize;
                if flags & FLAG_TRAN_LINEAR_VEC != 0 {
                    x += cached_args[arg_start + slice];
                    y += cached_args[arg_start + slice + 1];
                    px += cached_args[arg_start + slice + 2];
                    py += cached_args[arg_start + slice + 3];
                    slice += 4;
                }
                if flags & FLAG_TRAN_LINEAR_MAT_XX != 0 {
                    x += x0 * cached_args[arg_start + slice]
                        + y0 * cached_args[arg_start + slice + 1];
                    y += x0 * cached_args[arg_start + slice + 2]
                        + y0 * cached_args[arg_start + slice + 3];
                    slice += 4;
                }
                if flags & FLAG_TRAN_LINEAR_MAT_PXX != 0 {
                    x += px0 * cached_args[arg_start + slice]
                        + py0 * cached_args[arg_start + slice + 1];
                    y += px0 * cached_args[arg_start + slice + 2]
                        + py0 * cached_args[arg_start + slice + 3];
                    slice += 4;
                }
                if flags & FLAG_TRAN_LINEAR_MAT_XPX != 0 {
                    px += x0 * cached_args[arg_start + slice]
                        + y0 * cached_args[arg_start + slice + 1];
                    py += x0 * cached_args[arg_start + slice + 2]
                        + y0 * cached_args[arg_start + slice + 3];
                    slice += 4;
                }
                if flags & FLAG_TRAN_LINEAR_MAT_PXPX != 0 {
                    px += px0 * cached_args[arg_start + slice]
                        + py0 * cached_args[arg_start + slice + 1];
                    py += px0 * cached_args[arg_start + slice + 2]
                        + py0 * cached_args[arg_start + slice + 3];
                    slice += 4;
                }
                if flags & FLAG_TRAN_LINEAR_DP != 0 {
                    x += dp0 * cached_args[arg_start + slice];
                    y += dp0 * cached_args[arg_start + slice + 1];
                    px += dp0 * cached_args[arg_start + slice + 2];
                    py += dp0 * cached_args[arg_start + slice + 3];
                }
            } else if op == OP_SET_APERTURE as u32 {
                aperture_sq = cached_args[arg_start];
            } else if op == OP_REWIND as u32 {
                turn += 1;
                inst_current = 0;
            }

            if op != OP_REWIND as u32
                && op != OP_DUMP as u32
                && !(op == OP_TRAN_LINEAR as u32 && (flags & FLAG_TRAN_LINEAR_NO_PASS) != 0)
            {
                if flags & FLAG_NO_APERTURE_CHECK == 0 {
                    let center_distance = x * x + y * y;
                    alive *= F::cast_from(center_distance < aperture_sq);
                }
                passed_elements += alive;
            }
        }
    }
}

#[cube]
fn drift<F: Float>(flags: u32, length: F, x: &mut F, y: &mut F, px: F, py: F, opdp: F) {
    let mut eff_length = length;
    if flags & FLAG_EXACT != 0 {
        eff_length *= (opdp * opdp - px * px - py * py).inverse_sqrt();
    }
    *x += px * eff_length;
    *y += py * eff_length;
}

#[cube]
fn kick<F: Float>(
    flags: u32,
    knl_size: u32,
    ksl_size: u32,
    arg_start: usize,
    args: &SharedMemory<F>,
    x: F,
    y: F,
    oodppo: F,
    px: &mut F,
    py: &mut F,
) {
    if knl_size > 0 {
        let mut order = knl_size - 1;
        let mut dpx = args[arg_start + order as usize];
        dpx = if flags & FLAG_ACHROMATIC != 0 {
            dpx
        } else {
            dpx * oodppo
        };
        let mut dpy = F::new(0.0);
        while order > 0 {
            let order_f = F::cast_from(order);
            let aux = (dpx * x - dpy * y) / order_f;
            dpy = (dpx * y + dpy * x) / order_f;
            dpx = args[arg_start + order as usize - 1];
            dpx = if flags & FLAG_ACHROMATIC != 0 {
                dpx
            } else {
                dpx * oodppo
            };
            dpx += aux;
            order -= 1;
        }
        *px -= dpx;
        *py += dpy;
    }

    if ksl_size > 0 {
        let ksl_start = arg_start + knl_size as usize;
        let mut order = ksl_size - 1;
        let mut dpy = args[ksl_start + order as usize];
        dpy = if flags & FLAG_ACHROMATIC != 0 {
            dpy
        } else {
            dpy * oodppo
        };
        let mut dpx = F::new(0.0);
        while order > 0 {
            let order_f = F::cast_from(order);
            let aux = (dpx * y + dpy * x) / order_f;
            dpx = (dpx * x - dpy * y) / order_f;
            dpy = args[ksl_start + order as usize - 1];
            dpy = if flags & FLAG_ACHROMATIC != 0 {
                dpy
            } else {
                dpy * oodppo
            };
            dpy += aux;
            order -= 1;
        }
        *px -= dpx;
        *py += dpy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bytecode, PassFlags};

    #[test]
    fn decodes_headers_offsets_and_arguments() {
        let bytecode = Bytecode::from_instructions(
            vec![
                Instruction::drift("d", 2.0, true),
                Instruction::kick("k", vec![0.1], vec![0.2], false, true),
                Instruction::rewind("rw"),
            ],
            false,
        );

        let decoded = decode_program(bytecode.instructions());

        assert_eq!(decoded.headers.len(), 3);
        assert_eq!(decoded.arg_offsets, vec![0, 1, 3]);
        assert_eq!(decoded.args32, vec![2.0, 0.1, 0.2]);
        assert_eq!(decoded.args64, vec![2.0, 0.1, 0.2]);

        let drift = decoded.headers[0];
        assert_eq!(header_field(drift, HEADER_OP_SHIFT), OP_DRIFT as u32);
        assert_eq!(
            header_field(drift, HEADER_FLAGS_SHIFT),
            PassFlags::EXACT.bits() as u32
        );
        assert_eq!(header_field(drift, HEADER_ARGC_SHIFT), 1);

        let kick = decoded.headers[1];
        assert_eq!(header_field(kick, HEADER_OP_SHIFT), OP_KICK as u32);
        assert_eq!(header_field(kick, HEADER_AUX_SHIFT), 1);
        assert_eq!(header_field(kick, HEADER_ARGC_SHIFT), 2);
    }

    #[test]
    fn particle_packing_roundtrips_f32_and_f64() {
        let particles = vec![Particle {
            x: 1.25,
            y: -0.5,
            z: 0.125,
            px: 0.25,
            py: -0.75,
            dp: 0.01,
            passed_elements: 7,
            alive: false,
        }];

        let packed32 = f32::pack_particles(&particles);
        let unpacked32 = f32::unpack_particles(&packed32);
        assert_eq!(unpacked32[0].x, 1.25);
        assert_eq!(unpacked32[0].passed_elements, 7);
        assert!(!unpacked32[0].alive);

        let packed64 = f64::pack_particles(&particles);
        let unpacked64 = f64::unpack_particles(&packed64);
        assert_eq!(unpacked64, particles);
    }
}
